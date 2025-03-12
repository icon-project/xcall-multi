// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import {console2} from "forge-std/Test.sol";

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@xcall/utils/Types.sol";
import "@xcall/contracts/xcall/interfaces/IConnection.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/ICallService.sol";
import "@iconfoundation/xcall-solidity-library/utils/RLPEncode.sol";
import "@iconfoundation/xcall-solidity-library/utils/RLPEncode.sol";
import "@iconfoundation/xcall-solidity-library/utils/Strings.sol";
import "@iconfoundation/xcall-solidity-library/utils/Integers.sol";

/// @custom:oz-upgrades-from contracts/adapters/ClusterConnectionV1.sol:ClusterConnectionV1
contract ClusterConnection is Initializable, IConnection {
    using RLPEncode for bytes;
    using RLPEncode for string;
    using RLPEncode for uint256;

    using Strings for bytes;
    using Integers for uint256;

    mapping(string => uint256) private messageFees;
    mapping(string => uint256) private responseFees;
    mapping(string => mapping(uint256 => bool)) receipts;

    address private xCall;
    address private relayerAddress;
    address private adminAddress;
    uint256 public connSn;
    address[] private validators;
    uint8 private validatorsThreshold;

    event Message(string targetNetwork, uint256 sn, bytes _msg);
    event ValidatorSetAdded(bytes[] _validator, uint8 _threshold);

    modifier onlyRelayer() {
        require(msg.sender == this.relayer(), "OnlyRelayer");
        _;
    }

    modifier onlyAdmin() {
        require(msg.sender == this.admin(), "OnlyAdmin");
        _;
    }

    function initialize(address _relayer, address _xCall) public initializer {
        xCall = _xCall;
        adminAddress = msg.sender;
        relayerAddress = _relayer;
    }

    function listValidators() external view returns (address[] memory) {
        return validators;
    }

    function updateValidators(
        bytes[] memory _validators,
        uint8 _threshold
    ) external onlyAdmin {
        delete validators;
        for (uint i = 0; i < _validators.length; i++) {
            address validators_address = publicKeyToAddress(_validators[i]);
            if (
                !isValidator(validators_address) &&
                validators_address != address(0)
            ) {
                validators.push(validators_address);
            }
        }
        require(validators.length >= _threshold, "Not enough validators");
        validatorsThreshold = _threshold;
        emit ValidatorSetAdded(_validators, _threshold);
    }

    function isValidator(address signer) public view returns (bool) {
        for (uint i = 0; i < validators.length; i++) {
            if (validators[i] == signer) {
                return true;
            }
        }
    }

    /**
     @notice Sets the fee to the target network
     @param networkId String Network Id of target chain
     @param messageFee Integer ( The fee needed to send a Message )
     @param responseFee Integer (The fee of the response )
     */
    function setFee(
        string calldata networkId,
        uint256 messageFee,
        uint256 responseFee
    ) external onlyRelayer {
        messageFees[networkId] = messageFee;
        responseFees[networkId] = responseFee;
    }

    /**
     @notice Gets the fee to the target network
    @param to String Network Id of target chain
    @param response Boolean ( Whether the responding fee is included )
    @return fee Integer (The fee of sending a message to a given destination network )
    */
    function getFee(
        string memory to,
        bool response
    ) external view returns (uint256 fee) {
        uint256 messageFee = messageFees[to];
        if (response == true) {
            uint256 responseFee = responseFees[to];
            return messageFee + responseFee;
        }
        return messageFee;
    }

    /**
     @notice Sends the message to a specific network.
     @param sn : positive for two-way message, zero for one-way message, negative for response
     @param to  String ( Network Id of destination network )
     @param _svc String ( name of the service )
     @param sn  Integer ( serial number of the xcall message )
     @param _msg Bytes ( serialized bytes of Service Message )
     */
    function sendMessage(
        string calldata to,
        string calldata _svc,
        int256 sn,
        bytes calldata _msg
    ) external payable override {
        require(msg.sender == xCall, "Only Xcall can call sendMessage");
        uint256 fee;
        if (sn > 0) {
            fee = this.getFee(to, true);
        } else if (sn == 0) {
            fee = this.getFee(to, false);
        }
        require(msg.value >= fee, "Fee is not Sufficient");
        connSn++;
        emit Message(to, connSn, _msg);
    }

    /**
     @notice Sends the message to a xCall.
     @param srcNetwork  String ( Network Id )
     @param _connSn Integer ( connection message sn )
     @param _msg Bytes ( serialized bytes of Service Message )
     */
    function recvMessageWithSignatures(
        string memory srcNetwork,
        uint256 _connSn,
        bytes calldata _msg,
        bytes[] calldata _signedMessages
    ) public onlyRelayer {
        require(
            _signedMessages.length >= validatorsThreshold,
            "Not enough signatures passed"
        );

        string memory dstNetwork = ICallService(xCall).getNetworkId();

        bytes32 messageHash = getMessageHash(
            srcNetwork,
            _connSn,
            _msg,
            dstNetwork
        );
        uint signerCount = 0;
        address[] memory collectedSigners = new address[](
            _signedMessages.length
        );

        for (uint i = 0; i < _signedMessages.length; i++) {
            address signer = recoverSigner(messageHash, _signedMessages[i]);
            require(signer != address(0), "Invalid signature");
            if (
                !isValidatorProcessed(collectedSigners, signer) &&
                existsInValidators(signer)
            ) {
                collectedSigners[signerCount] = signer;
                signerCount++;
            }
        }
        require(
            signerCount >= validatorsThreshold,
            "Not enough valid signatures passed"
        );
        recvMessage(srcNetwork, _connSn, _msg);
    }

    function existsInValidators(address signer) internal view returns (bool) {
        for (uint i = 0; i < validators.length; i++) {
            if (validators[i] == signer) return true;
        }
        return false;
    }

    function isValidatorProcessed(
        address[] memory processedSigners,
        address signer
    ) public pure returns (bool) {
        for (uint i = 0; i < processedSigners.length; i++) {
            if (processedSigners[i] == signer) {
                return true;
            }
        }
        return false;
    }

    function recoverSigner(
        bytes32 messageHash,
        bytes memory signature
    ) public pure returns (address) {
        require(signature.length == 65, "Invalid signature length");
        bytes32 r;
        bytes32 s;
        uint8 v;
        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }
        if (v < 27) {
            v += 27;
        }
        require(v == 27 || v == 28, "Invalid signature 'v' value");
        return ecrecover(messageHash, v, r, s);
    }

    function recvMessage(
        string memory srcNetwork,
        uint256 _connSn,
        bytes calldata _msg
    ) internal {
        require(!receipts[srcNetwork][_connSn], "Duplicate Message");
        receipts[srcNetwork][_connSn] = true;
        ICallService(xCall).handleMessage(srcNetwork, _msg);
    }

    /**
     @notice Sends the balance of the contract to the owner(relayer)

    */
    function claimFees() public onlyRelayer {
        payable(relayerAddress).transfer(address(this).balance);
    }

    /**
     @notice Revert a messages, used in special cases where message can't just be dropped
     @param sn  Integer ( serial number of the  xcall message )
     */
    function revertMessage(uint256 sn) public onlyRelayer {
        ICallService(xCall).handleError(sn);
    }

    /**
     @notice Gets a message receipt
     @param srcNetwork String ( Network Id )
     @param _connSn Integer ( connection message sn )
     @return boolean if is has been recived or not
     */
    function getReceipt(
        string memory srcNetwork,
        uint256 _connSn
    ) public view returns (bool) {
        return receipts[srcNetwork][_connSn];
    }

    /**
        @notice Set the address of the admin.
        @param _address The address of the admin.
     */
    function setAdmin(address _address) external onlyAdmin {
        adminAddress = _address;
    }

    /**
        @notice Set the address of the relayer.
        @param _address The address of the relayer.
     */
    function setRelayer(address _address) external onlyAdmin {
        relayerAddress = _address;
    }

    /**
       @notice Gets the address of relayer
       @return (Address) the address of relayer
    */
    function relayer() external view returns (address) {
        return relayerAddress;
    }

    /**
        @notice Gets the address of admin
        @return (Address) the address of admin
     */
    function admin() external view returns (address) {
        return adminAddress;
    }

    /**
        @notice Set the required signature count for verification.
        @param _count The desired count.
     */
    function setRequiredValidatorCount(uint8 _count) external onlyAdmin {
        validatorsThreshold = _count;
    }

    function getRequiredValidatorCount() external view returns (uint8) {
        return validatorsThreshold;
    }

    function getMessageHash(
        string memory srcNetwork,
        uint256 _connSn,
        bytes calldata _msg,
        string memory dstNetwork
    ) internal pure returns (bytes32) {
        bytes memory encoded = abi
            .encodePacked(
                srcNetwork,
                _connSn.toString(),
                _msg,
                dstNetwork
            );
        return keccak256(encoded);
    }

    function publicKeyToAddress(
        bytes memory publicKey
    ) internal pure returns (address addr) {
        require(publicKey.length == 65, "Invalid public key length");

        bytes32 hash;

        assembly {
            let publicKeyStart := add(publicKey, 0x20)
            let destinationStart := add(publicKeyStart, 1)
            hash := keccak256(destinationStart, 64)
        }

        addr = address(uint160(uint256(hash)));
    }
}
