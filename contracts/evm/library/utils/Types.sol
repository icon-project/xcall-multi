// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;

/**
 * @notice List of ALL Struct being used to Encode and Decode RLP Messages
 */
library Types {
    // The name of CallService.
    string constant NAME = "xcallM";

    int constant CS_REQUEST = 1;
    int constant CS_RESULT = 2;

    int constant CALL_MESSAGE_TYPE = 1;
    int constant CALL_MESSAGE_ROLLBACK_TYPE = 2;
    int constant PERSISTENT_MESSAGE_TYPE = 3;

    struct RollbackData {
        address from;
        string to;
        string[] sources;
        bytes rollback;
        bool enabled; //whether wait response or received
    }

    struct CSMessage {
        int msgType;
        bytes payload;
    }

    struct CSMessageRequest {
        string from;
        string to;
        uint256 sn;
        int messageType;
        bytes data;
        string[] protocols;
    }

    struct ProxyRequest {
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

    function createPersistentMessageEnvelope(bytes memory data) internal pure returns (XCallEnvelope memory) {
        return XCallEnvelope(
            PERSISTENT_MESSAGE_TYPE,
            data,
            new string[](0),
            new string[](0)
        );
    }

}
