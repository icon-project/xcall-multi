use anchor_lang::prelude::*;

#[event]
pub struct MessageReceived {
    pub from: String,
    pub data: Vec<u8>,
}

#[event]
pub struct RollbackDataReceived {
    pub from: String,
    pub ssn: u128,
    pub rollback: Vec<u8>,
}
