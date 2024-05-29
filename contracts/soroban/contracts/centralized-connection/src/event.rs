#![allow(non_snake_case)]

use soroban_sdk::{contracttype, Bytes, Env, String};

#[contracttype]
pub struct SendMsgEvent {
    pub targetNetwork: String,
    pub connSn: u128,
    pub msg: Bytes,
}

pub(crate) fn send_message(e: &Env, targetNetwork: String, connSn: u128, msg: Bytes) {
    let emit_message = SendMsgEvent {
        targetNetwork,
        connSn,
        msg,
    };
    e.events().publish(("Message",), emit_message);
}
