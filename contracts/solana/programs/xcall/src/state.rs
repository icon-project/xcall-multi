use anchor_lang::prelude::*;

use crate::{
    error::XcallError,
    types::{request::CSMessageRequest, rollback::Rollback},
};

#[account]
#[derive(Debug)]
pub struct Config {
    pub admin: Pubkey,
    pub fee_handler: Pubkey,
    pub network_id: String,
    pub protocol_fee: u128,
    pub sequence_no: u128,
    pub last_req_id: u128,
}

impl Config {
    pub const SEED_PREFIX: &'static str = "config";

    pub fn new(admin: Pubkey, network_id: String) -> Self {
        Self {
            admin,
            fee_handler: admin,
            network_id,
            protocol_fee: 0,
            sequence_no: 0,
            last_req_id: 0,
        }
    }

    pub fn ensure_admin(&self, signer: Pubkey) -> Result<()> {
        if self.admin != signer {
            return Err(XcallError::OnlyAdmin.into());
        }
        Ok(())
    }

    pub fn ensure_fee_handler(&self, signer: Pubkey) -> Result<()> {
        if self.fee_handler != signer {
            return Err(XcallError::OnlyAdmin.into());
        }
        Ok(())
    }

    pub fn set_admin(&mut self, account: Pubkey) {
        self.admin = account
    }

    pub fn set_fee_handler(&mut self, fee_handler: Pubkey) {
        self.fee_handler = fee_handler
    }

    pub fn set_protocol_fee(&mut self, fee: u128) {
        self.protocol_fee = fee
    }

    pub fn get_next_sn(&mut self) -> u128 {
        self.sequence_no += 1;
        self.sequence_no
    }

    pub fn get_next_req_id(&mut self) -> u128 {
        self.last_req_id += 1;
        self.last_req_id
    }
}

#[account]
#[derive(InitSpace)]
pub struct DefaultConnection {
    pub address: Pubkey,
}

impl DefaultConnection {
    pub const SEED_PREFIX: &'static str = "conn";

    pub fn set(&mut self, address: Pubkey) {
        self.address = address
    }
}

#[account]
pub struct Reply {
    pub reply_state: Option<CSMessageRequest>,
    pub call_reply: Option<CSMessageRequest>,
}

impl Reply {
    pub const SEED_PREFIX: &'static str = "reply";
}

#[account]
pub struct RollbackAccount {
    pub rollback: Rollback,
    pub bump: u8,
}

impl RollbackAccount {
    pub const SEED_PREFIX: &'static str = "rollback";
}

#[account]
pub struct PendingRequest {
    pub sources: Vec<String>,
}

#[account]
#[derive(Debug)]
pub struct PendingResponse {
    pub sources: Vec<String>,
}

#[account]
pub struct SuccessfulResponse {
    pub success: bool,
}

#[account]
pub struct ProxyRequest {
    pub req: CSMessageRequest,
    pub bump: u8,
}
