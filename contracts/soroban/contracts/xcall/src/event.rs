#![allow(non_snake_case)]

use crate::types::result::CSResponseType;
use soroban_sdk::{contracttype, Address, Bytes, Env, String};

#[contracttype]
pub struct CallMsgSentEvent {
    pub from: Address,
    pub to: String,
    pub sn: u128,
}

#[contracttype]
pub struct CallMsgEvent {
    pub from: String,
    pub to: String,
    pub sn: u128,
    pub reqId: u128,
    pub data: Bytes,
}

#[contracttype]
pub struct ResponseMsgEvent {
    pub code: u32,
    pub sn: u128,
}

#[contracttype]
pub struct CallExecutedEvent {
    pub reqId: u128,
    pub code: u32,
    pub msg: String,
}

#[contracttype]
pub struct RollbackMsgEvent {
    pub sn: u128,
}

#[contracttype]
pub struct RollbackExecutedEvent {
    pub sn: u128,
}

pub(crate) fn message_sent(e: &Env, from: Address, to: String, sn: u128) {
    let data = CallMsgSentEvent { from, to, sn };
    e.events().publish(("CallMessageSent",), data)
}

pub(crate) fn call_message(e: &Env, from: String, to: String, sn: u128, reqId: u128, data: Bytes) {
    let data = CallMsgEvent {
        from,
        to,
        sn,
        reqId,
        data,
    };
    e.events().publish(("CallMessage",), data);
}

pub(crate) fn call_executed(e: &Env, reqId: u128, code: u8, msg: String) {
    let data = CallExecutedEvent {
        reqId,
        code: code as u32,
        msg,
    };
    e.events().publish(("CallExecuted",), data)
}

pub(crate) fn response_message(e: &Env, code: CSResponseType, sn: u128) {
    let response_code: u8 = code.into();
    let data = ResponseMsgEvent {
        code: response_code as u32,
        sn,
    };
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
