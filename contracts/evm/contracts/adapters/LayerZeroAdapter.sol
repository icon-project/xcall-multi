// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@xcall/utils/Types.sol";
import "@iconfoundation/btp2-solidity-library/interfaces/ICallService.sol";
import "@lz-contracts/interfaces/ILayerZeroReceiver.sol";
import "@lz-contracts/interfaces/ILayerZeroEndpoint.sol";
import "./interfaces/ILayerZeroAdapter.sol";
import "@xcall/contracts/xcall/interfaces/IConnection.sol";

/**
 * @title LayerZeroAdapter
 * @dev A contract serves as a cross-chain xcall adapter, enabling communication between xcall on different blockchain networks via LayerZero.
 */
contract LayerZeroAdapter is ILayerZeroAdapter, Initializable, ILayerZeroReceiver, IConnection {
    bytes constant private EMPTY_BYTES = new bytes(2048);
    mapping(uint256 => Types.PendingResponse) private pendingResponses;
    mapping(string => uint16) private chainIds;
    mapping(uint16 => string) private networkIds;
    mapping(string => bytes) private adapterParams;
    mapping(string => bytes) private remoteEndpoint;
    address private layerZeroEndpoint;
    address private xCall;
    address private owner;
    address private adminAddress;

    modifier onlyOwner() {
        require(msg.sender == owner, "OnlyOwner");
        _;
    }

    modifier onlyAdmin() {
        require(msg.sender == adminAddress, "OnlyAdmin");
        _;
    }

    /**
     * @dev Initializes the contract with LayerZero endpoint and xCall address.
     * @param _layerZeroEndpoint The address of the LayerZero endpoint contract.
     * @param _xCall The address of the xCall contract.
     */
    function initialize(address _layerZeroEndpoint, address _xCall) public initializer {
        owner = msg.sender;
        adminAddress = msg.sender;
        layerZeroEndpoint = _layerZeroEndpoint;
        xCall = _xCall;
    }

    /**
     * @dev Configure connection settings for a destination chain.
     * @param networkId The network ID of the destination chain.
     * @param chainId The chain ID of the destination chain.
     * @param endpoint The endpoint or address of the destination chain.
     * @param gasLimit The gas limit for the connection on the destination chain.
     */
    function configureConnection(
        string memory networkId,
        uint16 chainId,
        bytes memory endpoint,
        uint256 gasLimit
    ) external override onlyAdmin {
        require(bytes(networkIds[chainId]).length == 0, "Connection already configured");
        networkIds[chainId] = networkId;
        chainIds[networkId] = chainId;
        remoteEndpoint[networkId] = abi.encodePacked(endpoint, address(this));
        if (gasLimit > 0) {
            adapterParams[networkId] = abi.encodePacked(uint16(1), gasLimit);
        } else {
            adapterParams[networkId] = bytes("");
        }
    }

    /**
     * @notice set or update gas limit for a destination chain.
     * @param networkId The network ID of the destination chain.
     * @param gasLimit The gas limit for transactions on the destination chain.
     */
    function setGasLimit(
        string calldata networkId,
        uint256 gasLimit
    ) external override onlyAdmin {
        if (gasLimit > 0) {
            adapterParams[networkId] = abi.encodePacked(uint16(1), gasLimit);
        } else {
            adapterParams[networkId] = bytes("");
        }
    }

    /**
     * @dev Get the gas fee required to send a message to a specified destination network.
     * @param _to The network ID of the target chain.
     * @param _response Indicates whether the response fee is included (true) or not (false).
     * @return _fee The fee for sending a message to the given destination network.
     */
    function getFee(string memory _to, bool _response) external view override returns (uint256 _fee) {
        (_fee,) = ILayerZeroEndpoint(layerZeroEndpoint).estimateFees(chainIds[_to], address(this), EMPTY_BYTES, false, adapterParams[_to]);
    }

    /**
     * @dev Send a message to a specified destination network.
     * @param _to The network ID of the destination network.
     * @param _svc The name of the service.
     * @param _sn The serial number of the message.
     * @param _msg The serialized bytes of the service message.
     */
    function sendMessage(
        string memory _to,
        string memory _svc,
        int256 _sn,
        bytes memory _msg
    ) external override payable {
        require(msg.sender == xCall, "Only xCall can send messages");
        uint256 fee;

        if (_sn < 0) {
            fee = this.getFee(_to, false);
            if (address(this).balance < fee) {
                uint256 sn = uint256(- _sn);
                pendingResponses[sn] = Types.PendingResponse(_msg, _to);
                emit ResponseOnHold(sn);
                return;
            }
        } else {
            fee = msg.value;
        }


        ILayerZeroEndpoint(layerZeroEndpoint).send{value: fee}(
            chainIds[_to],
            remoteEndpoint[_to],
            abi.encodePacked(_msg),
            payable(address(this)),
            address(0x0),
            adapterParams[_to]
        );
    }

    /**
     * @dev Endpoint that the LayerZero Relayer contract calls to deliver the payload.
     * @param sourceChain The source chain ID.
     * @param _srcAddress The source address.
     * @param _nonce The nonce.
     * @param payload The payload to be delivered.
     */
    function lzReceive(
        uint16 sourceChain,
        bytes memory _srcAddress,
        uint64 _nonce,
        bytes memory payload
    ) public override {
        require(msg.sender == layerZeroEndpoint, "Invalid endpoint caller");
        string memory nid = networkIds[sourceChain];
        require(keccak256(_srcAddress) == keccak256(abi.encodePacked(remoteEndpoint[nid])), "Source address mismatched");
        ICallService(xCall).handleMessage(nid, payload);
    }

    /**
     * @dev Pay and trigger the execution of a stored response to be sent back.
     * @param _sn The serial number of the message for which the response is being triggered.
     */
    function triggerResponse(uint256 _sn) external override payable {
        int256 sn = int256(_sn);
        Types.PendingResponse memory resp = pendingResponses[_sn];
        delete pendingResponses[_sn];
        uint256 fee = msg.value;

        ILayerZeroEndpoint(layerZeroEndpoint).send{value: fee}(
            chainIds[resp.targetNetwork],
            remoteEndpoint[resp.targetNetwork],
            abi.encodePacked(resp.msg),
            payable(address(this)),
            address(0x0),
            adapterParams[resp.targetNetwork]
        );
    }

    /**
     * @dev Set the address of the admin.
     * @param _address The address of the admin.
     */
    function setAdmin(address _address) external onlyAdmin {
        adminAddress = _address;
    }

    /**
     * @dev Get the address of the admin.
     * @return (Address) The address of the admin.
     */
    function admin() external view returns (address) {
        if (adminAddress == address(0)) {
            return owner;
        }
        return adminAddress;
    }

    fallback() external payable {}
}
