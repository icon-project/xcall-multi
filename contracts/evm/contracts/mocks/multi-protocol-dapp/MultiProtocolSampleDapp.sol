// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "@iconfoundation/xcall-solidity-library/utils/NetworkAddress.sol";
import "@iconfoundation/xcall-solidity-library/utils/Integers.sol";
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";
import "@iconfoundation/xcall-solidity-library/utils/Strings.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/ICallService.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/ICallServiceReceiver.sol";
import "@xcall/utils/Types.sol";

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";

contract MultiProtocolSampleDapp is Initializable, ICallServiceReceiver {
    using Strings for string;
    using Integers for uint;
    using ParseAddress for address;
    using ParseAddress for string;
    using NetworkAddress for string;

    address private callSvc;
    mapping(string => string[]) private sources;
    mapping(string => string[]) private destinations;

    event MessageReceived(string indexed from, bytes data);

    function initialize(address _callService) public initializer {
        callSvc = _callService;
    }

    modifier onlyCallService() {
        require(msg.sender == callSvc, "onlyCallService");
        _;
    }

    function addConnection(string memory nid, string memory source, string memory destination) external {
        sources[nid].push(source);
        destinations[nid].push(destination);
    }

    function getSources(string memory nid) public view returns (string[] memory) {
        return sources[nid];
    }

    function getDestinations(string memory nid) public view returns (string[] memory) {
        return destinations[nid];
    }

    function sendMessage(string memory to, bytes memory data, bytes memory rollback) external payable {
        _sendCallMessage(msg.value, to, data, rollback);
    }

    function sendNewMessage(string memory to, bytes memory data, int256 messageType, bytes memory rollback) external payable {
        
        bytes memory message;
        (string memory net,) = to.parseNetworkAddress();
        string[] memory _sources = getSources(net);
        string[] memory _destinations = getDestinations(net);

        if (messageType == Types.PERSISTENT_MESSAGE_TYPE) {
            message = Types.createPersistentMessage(data, _sources, _destinations);
        } else if(messageType == Types.CALL_MESSAGE_TYPE) {
            message = Types.createCallMessage(data, _sources, _destinations);
        } else if(messageType == Types.CALL_MESSAGE_ROLLBACK_TYPE) {
            require(rollback.length > 0, "InvalidRollback");
            message = Types.createCallMessageWithRollback(data, rollback, _sources, _destinations);
        } else {
            revert("InvalidMessageType");
        }
        _sendCall(msg.value, to, message);
    }

    function sendMessageAny(string memory to, bytes memory data) external payable {
        _sendCall(msg.value, to, data);
    }

    function _sendCallMessage(
        uint256 value,
        string memory to,
        bytes memory data,
        bytes memory rollback
    ) private {
        (string memory net,) = to.parseNetworkAddress();
        ICallService(callSvc).sendCallMessage{value: value}(to, data, rollback, getSources(net), getDestinations(net));
    }

    function _sendCall(
        uint256 value,
        string memory to,
        bytes memory message
    ) private {
        ICallService(callSvc).sendCall{value: value}(to, message);
    }


    function handleCallMessage(string memory from, bytes memory data, string[] memory protocols) external onlyCallService {
        (string memory netFrom,) = from.parseNetworkAddress();
        string memory rollbackAddress = ICallService(callSvc).getNetworkAddress();

        if (from.compareTo(rollbackAddress)) {
            return;
        } else {
            require(protocolsEqual(protocols, getSources(netFrom)), "invalid protocols");
            require(keccak256(data) != keccak256(abi.encodePacked("rollback")), "rollback");

            if(keccak256(data) == keccak256(abi.encodePacked("reply-reponse"))) {
                _sendCallMessage(0, from, '010203', bytes(""));
            }
            emit MessageReceived(from, data);
        }
    }

    function protocolsEqual(string[] memory a, string[] memory b) private pure returns (bool) {
        if (a.length != b.length) {
            return false;
        }

        for (uint256 i = 0; i < a.length; i++) {
            if (keccak256(abi.encodePacked(a[i])) != keccak256(abi.encodePacked(b[i]))) {
                return false;
            }
        }

        return true;
    }

    function isEqual(bytes memory a, string memory b) private pure returns (bool) {
        return keccak256(a) == keccak256(abi.encodePacked(b));
    }
}
