// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

library Types {
    /// @title SwapOrder type
    /// @notice Represents a swap order with source and destination network identifiers, creator, destination address, token details, and minimum receive amount.
    struct SwapOrder {
        uint256 id; // unique ID
        string emitter; // Address of emitter contract
        string srcNID; // Source Network ID
        string dstNID; // Destination Network ID
        string creator; // The user who created the order
        string destinationAddress; // Destination address on the destination network
        string token; // Token to be swapped
        uint256 amount; // Amount of the token to be swapped
        string toToken; // Token to receive on the destination network
        uint256 toAmount; // Minimum amount of the toToken to receive
        bytes data; // Additional data (if any) for future use
    }

    uint constant FILL = 1; // Constant for Fill message type
    uint constant CANCEL = 2; // Constant for Cancel message type

    struct OrderMessage {
        uint messageType; // Message type: FILL or CANCEL
        bytes message; // Encoded message content
    }

    /// @title OrderFill type
    /// @notice Represents an order fill with the corresponding order ID, order hash, solver address, and fill amount.
    struct OrderFill {
        uint256 id; // ID of the order being filled
        bytes orderBytes; // rlp of the order
        string solver; // Address of the solver that fills the order
    }

    /// @title Cancel type
    /// @notice Represents a cancellation of an order with the corresponding order.
    struct Cancel {
        bytes orderBytes; // rlp encoded  order to be canceled
    }
}
