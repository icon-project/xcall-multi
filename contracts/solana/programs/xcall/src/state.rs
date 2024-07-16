use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::XcallError,
    types::{request::CSMessageRequest, rollback::Rollback},
};

#[account]
#[derive(Debug)]
pub struct Config {
    pub admin: Pubkey,
    pub fee_handler: Pubkey,
    pub network_id: String,
    pub protocol_fee: u64,
    pub sequence_no: u128,
    pub last_req_id: u128,
    pub bump: u8,
}

impl Config {
    pub const SEED_PREFIX: &'static str = "config";

    pub const SIZE: usize = 8 + 1048 + 1;

    pub fn set(&mut self, admin: Pubkey, network_id: String, bump: u8) {
        self.admin = admin;
        self.bump = bump;
        self.fee_handler = admin;
        self.network_id = network_id;
        self.protocol_fee = 0;
        self.sequence_no = 0;
        self.last_req_id = 0;
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

    pub fn set_protocol_fee(&mut self, fee: u64) {
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
pub struct DefaultConnection {
    pub address: Pubkey,
    pub bump: u8,
}

impl DefaultConnection {
    pub const SEED_PREFIX: &'static str = "conn";

    pub const SIZE: usize = 8 + 32 + 1;

    pub fn set(&mut self, address: Pubkey, bump: u8) {
        self.address = address;
        self.bump = bump
    }
}

#[account]
pub struct Reply {
    pub reply_state: Option<CSMessageRequest>,
    pub call_reply: Option<CSMessageRequest>,
    pub bump: u8,
}

impl Reply {
    pub const SEED_PREFIX: &'static str = "reply";

    pub const SIZE: usize = 8 + 1024 + 1024 + 1;

    pub fn new(&mut self, bump: u8) {
        self.reply_state = None;
        self.call_reply = None;
        self.bump = bump
    }

    pub fn set_reply_state(&mut self, req: Option<CSMessageRequest>) {
        self.reply_state = req;
    }

    pub fn set_call_reply(&mut self, req: Option<CSMessageRequest>) {
        self.call_reply = req
    }
}

#[derive(Debug)]
#[account]
pub struct RollbackAccount {
    pub rollback: Rollback,
    pub creator_key: Pubkey,
    pub bump: u8,
}

impl RollbackAccount {
    pub const SEED_PREFIX: &'static str = "rollback";

    pub const SIZE: usize = 8 + 1024 + 1;

    pub fn set(&mut self, rollback: Rollback, creator_key: Pubkey, bump: u8) {
        self.rollback = rollback;
        self.creator_key = creator_key;
        self.bump = bump
    }
}

#[account]
#[derive(Debug)]
pub struct PendingRequest {
    pub sources: Vec<Pubkey>,
}

impl PendingRequest {
    pub const SEED_PREFIX: &'static str = "req";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 640;
}

#[account]
#[derive(Debug)]
pub struct PendingResponse {
    pub sources: Vec<Pubkey>,
}

impl PendingResponse {
    pub const SEED_PREFIX: &'static str = "res";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 640;
}

#[account]
pub struct SuccessfulResponse {
    pub success: bool,
}

impl SuccessfulResponse {
    pub const SEED_PREFIX: &'static str = "success";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 1;
}

#[account]
pub struct ProxyRequest {
    pub req: CSMessageRequest,
    pub creator_key: Pubkey,
    pub bump: u8,
}

impl ProxyRequest {
    pub const SEED_PREFIX: &'static str = "proxy";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 1024 + 32 + 1;

    pub fn set(&mut self, req: CSMessageRequest, creator_key: Pubkey, bump: u8) {
        self.req = req;
        self.creator_key = creator_key;
        self.bump = bump
    }
}
