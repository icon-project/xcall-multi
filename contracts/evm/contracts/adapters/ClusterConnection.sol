// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@xcall/utils/Types.sol";
import "@xcall/contracts/xcall/interfaces/IConnection.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/ICallService.sol";

contract ClusterConnection is Initializable, IConnection {
    mapping(string => uint256) private messageFees;
    mapping(string => uint256) private responseFees;
    mapping(string => mapping(uint256 => bool)) receipts;
    mapping(address => bool) public isSigner;

    address private xCall;
    address private adminAddress;
    uint256 public connSn;
    address[] private signers;
    uint8 private reqSigCount;

    event Message(string targetNetwork, uint256 sn, bytes _msg);
    event SignerAdded(address _signer);
    event SignerRemoved(address _signer);

    modifier onlyAdmin() {
        require(msg.sender == this.admin(), "OnlyRelayer");
        _;
    }

    function initialize(address _relayer, address _xCall) public initializer {
        xCall = _xCall;
        signers.push(_relayer);
        adminAddress = _relayer;
        emit SignerAdded(_relayer);
    }

    function listSigners() external view returns (address[] memory) {
        return signers;
    }

    function addSigner(address _newSigner) external onlyAdmin {
        require(!isSigner[_newSigner], "Address is already an signer");
        signers.push(_newSigner);
        isSigner[_newSigner] = true;
        emit SignerAdded(_newSigner);
    }

    // Function to remove an existing signer
    function removeSigner(address _signer) external onlyAdmin {
        require(_signer!=this.admin(), "cannot remove admin");
        for (uint i = 0; i < signers.length; i++) {
            if (signers[i] == _signer) {
                signers[i] = signers[signers.length - 1]; 
                signers.pop();                 
                isSigner[_signer] = false;
                break;
            }
            emit SignerRemoved(_signer);
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
    ) external onlyAdmin {
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
     @param svc String ( name of the service )
     @param sn  Integer ( serial number of the xcall message )
     @param _msg Bytes ( serialized bytes of Service Message )
     */
    function sendMessage(
        string calldata to,
        string calldata svc,
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
    ) public onlyAdmin {
        require(_signedMessages.length > 0, "No signatures provided");
        require(_signedMessages.length >= reqSigCount, "Not enough signatures passed");
        bytes32 messageHash = keccak256(_msg);
        uint signerCount = 0;
        address[] memory collectedSigners = new address[](_signedMessages.length);
        for (uint i = 0; i < _signedMessages.length; i++) {
            address signer = recoverSigner(messageHash, _signedMessages[i]);
            require(signer != address(0), "Invalid signature");
            if (!isSignerProcessed(collectedSigners, signer)){
                collectedSigners[signerCount] = signer;
                signerCount++;
            }
        }

        if (signerCount >= reqSigCount) {
            recvMessage(srcNetwork,_connSn,_msg);
        }
    }

    function isSignerProcessed(address[] memory processedSigners, address signer) public pure returns (bool) {
        for (uint i = 0; i < processedSigners.length; i++) {
            if (processedSigners[i] == signer) {
                return true;
            }
        }
        return false;
    }    

    function recoverSigner(bytes32 messageHash, bytes memory signature) public pure returns (address) {
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
        return ecrecover(toEthSignedMessageHash(messageHash), v, r, s);
    }

    function toEthSignedMessageHash(bytes32 _messageHash) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", _messageHash));
    }


    /**
     @notice Sends the message to a xCall.
     @param srcNetwork  String ( Network Id )
     @param _connSn Integer ( connection message sn )
     @param _msg Bytes ( serialized bytes of Service Message )
     */
    function recvMessage(
        string memory srcNetwork,
        uint256 _connSn,
        bytes calldata _msg
    ) public onlyAdmin {
        require(!receipts[srcNetwork][_connSn], "Duplicate Message");
        receipts[srcNetwork][_connSn] = true;
        ICallService(xCall).handleMessage(srcNetwork, _msg);
    }

    /**
     @notice Sends the balance of the contract to the owner(relayer)

    */
    function claimFees() public onlyAdmin {
        payable(adminAddress).transfer(address(this).balance);
    }

    /**
     @notice Revert a messages, used in special cases where message can't just be dropped
     @param sn  Integer ( serial number of the  xcall message )
     */
    function revertMessage(uint256 sn) public onlyAdmin {
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
    function setRequiredCount(uint8 _count) external onlyAdmin {
        reqSigCount = _count;
    }

    function getRequiredCount() external view returns (uint8) {
        return reqSigCount;
    }
}
