// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@xcall/utils/Types.sol";
import "@xcall/contracts/xcall/interfaces/IConnection.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/ICallService.sol";
import "@iconfoundation/xcall-solidity-library/utils/RLPEncode.sol";

contract ClusterConnection is Initializable, IConnection {

    using RLPEncode for bytes;
    using RLPEncode for string;
    using RLPEncode for uint256;

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
    event ValidatorSetAdded(address[] _validator, uint8 _threshold);

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

    function setValidators(address[] memory _validators, uint8 _threshold) external onlyAdmin {
        delete validators;
        for (uint i = 0; i < _validators.length; i++) {
            if(!isValidator(_validators[i]) && _validators[i] != address(0)) {
                validators.push(_validators[i]);   
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
        require(_signedMessages.length >= validatorsThreshold, "Not enough signatures passed");
        bytes32 messageHash = getMessageHash(srcNetwork, _connSn, _msg);
        uint signerCount = 0;
        address[] memory collectedSigners = new address[](_signedMessages.length);
        for (uint i = 0; i < _signedMessages.length; i++) {
            address signer = recoverSigner(messageHash, _signedMessages[i]);
            require(signer != address(0), "Invalid signature");
            if (!isValidatorProcessed(collectedSigners, signer)){
                collectedSigners[signerCount] = signer;
                signerCount++;
            }
        }
        require(signerCount >= validatorsThreshold,"Not enough valid signatures passed");
        recvMessage(srcNetwork,_connSn,_msg);
    }

    function isValidatorProcessed(address[] memory processedSigners, address signer) public pure returns (bool) {
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
    ) public onlyRelayer {
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
        @notice Set the address of the relayer.
        @param _address The address of the relayer.
     */
    function setAdmin(address _address) external onlyRelayer {
        adminAddress = _address;
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
    function setRequiredValidatorCount(uint8 _count) external onlyAdmin() {
        validatorsThreshold = _count;
    }

    function getRequiredValidatorCount() external view returns (uint8) {
        return validatorsThreshold;
    }

    function getMessageHash(string memory srcNetwork, uint256 _connSn, bytes calldata _msg) internal pure returns (bytes32) {
        bytes memory rlp = abi.encodePacked(
            srcNetwork.encodeString(),
            _connSn.encodeUint(),
            _msg.encodeBytes()
        );
        return keccak256(rlp);
    }
}
