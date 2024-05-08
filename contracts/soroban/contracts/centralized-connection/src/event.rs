use soroban_sdk::{Bytes, Env, String};

use crate::types::SendMsgEvent;

pub(crate) fn send_message(e: &Env, to: String, sn: u128, msg: Bytes) {
    let emit_message = SendMsgEvent {
        target_network: to,
        conn_sn: sn,
        msg,
    };
    e.events().publish(("EmitMessage",), emit_message);
}
