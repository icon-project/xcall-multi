#![allow(non_snake_case)]

use anchor_lang::prelude::*;

#[event]
pub struct CallMessageSent {
    pub from: Pubkey,
    pub to: String,
    pub sn: u128,
}

#[event]
pub struct CallMessage {
    pub from: String,
    pub to: String,
    pub sn: u128,
    pub reqId: u128,
    pub data: Vec<u8>,
}

#[event]
pub struct CallExecuted {
    pub reqId: u128,
    pub code: u8,
    pub msg: String,
}

#[event]
pub struct ResponseMessage {
    pub code: u8,
    pub sn: u128,
}

#[event]
pub struct RollbackMessage {
    pub sn: u128,
}

#[event]
pub struct RollbackExecuted {
    pub sn: u128,
}
