// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
import "@xcall/utils/RLPEncodeStruct.sol";
import "@xcall/utils/RLPEncodeStruct.sol";

/**
 * @notice List of ALL Struct being used to Encode and Decode RLP Messages
 */
library Types {
    using RLPEncodeStruct for Types.CallMessageWithRollback;
    using RLPEncodeStruct for Types.XCallEnvelope;

    // The name of CallService.
    string constant NAME = "xcallM";

    int constant CS_REQUEST = 1;
    /**
     * Legacy Code, CS_RESPONSE replaced by CS_RESULT in V2
     */
    int constant CS_RESPONSE = 2;

    int constant CS_RESULT = 2;

    int constant CALL_MESSAGE_TYPE = 0;
    int constant CALL_MESSAGE_ROLLBACK_TYPE = 1;
    int constant PERSISTENT_MESSAGE_TYPE = 2;

    /**
     * Legacy Code, CallRequest replaced with RollbackData
     */
    struct CallRequest {
        address from;
        string to;
        string[] sources;
        bytes rollback;
        bool enabled; //whether wait response or received
    }

    struct RollbackData {
        address from;
        string to;
        string[] sources;
        bytes rollback;
        bool enabled; 
    }

    struct CSMessage {
        int msgType;
        bytes payload;
    }

    struct CSMessageResponse {
        uint256 sn;
        int code;
    }

    /**
     * Legacy Code, CSMessageRequest replaced with CSMessageRequestV2
     */
    struct CSMessageRequest {
        string from;
        string to;
        uint256 sn;
        bool rollback;
        bytes data;
        string[] protocols;
    }

    /**
     * Legacy Code, ProxyRequest replaced with ProxyRequestV2
     */
    struct ProxyRequest {
        string from;
        string to;
        uint256 sn;
        bool rollback;
        bytes32 hash;
        string[] protocols;
    }

    struct CSMessageRequestV2 {
        string from;
        string to;
        uint256 sn;
        int messageType;
        bytes data;
        string[] protocols;
    }

    struct ProxyRequestV2 {
        string from;
        string to;
        uint256 sn;
        int256 messageType;
        bytes32 hash;
        string[] protocols;
    }

    int constant CS_RESP_SUCCESS = 1;
    int constant CS_RESP_FAILURE = 0;

    struct CSMessageResult {
        uint256 sn;
        int code;
        bytes message;
    }

    struct PendingResponse {
        bytes msg;
        string targetNetwork;
    }

    struct XCallEnvelope {
        int messageType;
        bytes message;
        string[] sources;
        string[] destinations;
    }

    struct CallMessage {
        bytes data;
    }

    struct CallMessageWithRollback {
        bytes data;
        bytes rollback;
    }

    struct ProcessResult {
        bool needResponse;
        bytes data;
    }

    function createPersistentMessage(
        bytes memory data,
        string[] memory sources,
        string[] memory destinations
    ) internal pure returns (bytes memory) {
        return
            XCallEnvelope(PERSISTENT_MESSAGE_TYPE, data, sources, destinations).encodeXCallEnvelope();
    }

    function createCallMessage(
        bytes memory data,
        string[] memory sources,
        string[] memory destinations
    ) internal pure returns (bytes memory) {
        return XCallEnvelope(CALL_MESSAGE_TYPE, data, sources, destinations).encodeXCallEnvelope();
    }

    function createCallMessageWithRollback(
        bytes memory data,
        bytes memory rollback,
        string[] memory sources,
        string[] memory destinations
    ) internal pure returns (bytes memory) {
        Types.CallMessageWithRollback memory _msg = Types
            .CallMessageWithRollback(data, rollback);

        return
            XCallEnvelope(
                CALL_MESSAGE_ROLLBACK_TYPE,
                _msg.encodeCallMessageWithRollback(),
                sources,
                destinations
            ).encodeXCallEnvelope();
    }
}
