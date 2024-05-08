use soroban_sdk::{contracttype, Address, Bytes, Env, String};

use crate::types::{network_address::NetworkAddress, result::CSResponseType};

#[contracttype]
pub struct CallMsgSentEvent {
    from: Address,
    to: NetworkAddress,
    sn: u128,
}

#[contracttype]
pub struct CallMsgEvent {
    pub from: NetworkAddress,
    pub to: String,
    pub sn: u128,
    pub req_id: u128,
    pub data: Bytes,
}

#[contracttype]
pub struct ResponseMsgEvent {
    code: CSResponseType,
    sn: u128,
}

#[contracttype]
pub struct CallExecutedEvent {
    req_id: u128,
    code: u32,
    msg: String,
}

#[contracttype]
pub struct RollbackMsgEvent {
    sn: u128,
}

#[contracttype]
pub struct RollbackExecutedEvent {
    sn: u128,
}

pub(crate) fn message_sent(e: &Env, from: Address, to: NetworkAddress, sn: u128) {
    let data = CallMsgSentEvent { from, to, sn };
    e.events().publish(("CallMessageSent",), data)
}

pub(crate) fn call_message(
    e: &Env,
    from: NetworkAddress,
    to: String,
    sn: u128,
    req_id: u128,
    data: Bytes,
) {
    let data = CallMsgEvent {
        from,
        to,
        sn,
        req_id,
        data,
    };
    e.events().publish(("CallMessage",), data);
}

pub(crate) fn call_executed(e: &Env, req_id: u128, code: u8, msg: String) {
    let data = CallExecutedEvent {
        req_id,
        code: code as u32,
        msg,
    };
    e.events().publish(("CallExecuted",), data)
}

pub(crate) fn response_message(e: &Env, code: CSResponseType, sn: u128) {
    let data = ResponseMsgEvent { code, sn };
    e.events().publish(("ResponseMessage",), data)
}

pub(crate) fn rollback_message(e: &Env, sn: u128) {
    let data = RollbackMsgEvent { sn };
    e.events().publish(("RollbackMessage",), data)
}

pub(crate) fn rollback_executed(e: &Env, sn: u128) {
    let data = RollbackExecutedEvent { sn };
    e.events().publish(("RollbackExecuted",), data)
}
