#![allow(non_snake_case)]

use soroban_sdk::{contracttype, Bytes, Env, String};

/// Emitted when a new swap intent is created
#[contracttype]
pub struct SwapIntent {
    // The ID of the swap order
    pub id: u128,
    // Address of emitter contract
    pub emitter: String,
    // The source network ID
    pub srcNID: String,
    // The destination network ID
    pub dstNID: String,
    // The address of the creator of the swap order
    pub creator: String,
    // The address where the swapped tokens will be sent
    pub destinationAddress: String,
    // The address of the token being swapped
    pub token: String,
    // The amount of token being swapped
    pub amount: u128,
    // The token to be received after the swap
    pub toToken: String,
    // The amount of tokens to be receive after the swap
    pub toAmount: u128,
    // Additional arbitrary data for the swap
    pub data: Bytes,
}

// Emitted when a swap order is filled
#[contracttype]
pub struct OrderFilled {
    // The ID of the order being filled
    pub id: u128,
    // The source network ID of the swap order
    pub srcNID: String,
}

// Emitted when a swap order is cancelled
#[contracttype]
pub struct OrderCancelled {
    // The ID of the order being cancelled
    pub id: u128,
    // The source network ID where the order was created
    pub srcNID: String,
}

// Emitted when a swap order is completed
#[contracttype]
pub struct OrderClosed {
    // The ID of the order
    pub id: u128,
}

/// Emitted when a cross-chain message is sent
#[contracttype]
pub struct Message {
    // The ID of the target network
    pub targetNetwork: String,
    // The connection sequence number
    pub sn: u128,
    // The rlp encoded message being sent to other chain
    pub msg: Bytes,
}

pub fn swap_intent(
    e: &Env,
    id: u128,
    emitter: String,
    srcNID: String,
    dstNID: String,
    creator: String,
    destinationAddress: String,
    token: String,
    amount: u128,
    toToken: String,
    toAmount: u128,
    data: Bytes,
) {
    let emit_message = SwapIntent {
        id,
        emitter,
        srcNID,
        dstNID,
        creator,
        destinationAddress,
        token,
        amount,
        toToken,
        toAmount,
        data,
    };
    e.events().publish(("SwapIntent",), emit_message);
}

pub fn order_filled(e: &Env, id: u128, srcNID: String) {
    let emit_message = OrderFilled { id, srcNID };

    e.events().publish(("OrderFilled",), emit_message);
}

pub fn order_closed(e: &Env, id: u128) {
    let emit_message = OrderClosed { id };

    e.events().publish(("OrderClosed",), emit_message);
}

pub fn order_cancelled(e: &Env, id: u128, srcNID: String) {
    let emit_message = OrderCancelled { id, srcNID };

    e.events().publish(("OrderCancelled",), emit_message);
}

pub fn send_message(e: &Env, targetNetwork: String, sn: u128, msg: Bytes) {
    let emit_message = Message {
        targetNetwork,
        sn,
        msg,
    };
    e.events().publish(("Message",), emit_message);
}
