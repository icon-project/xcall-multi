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

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 32 + 32 + 32 + 8 + 16 + 16 + 1;

    pub fn new(&mut self, admin: Pubkey, network_id: String, bump: u8) {
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

#[derive(Debug)]
#[account]
pub struct RollbackAccount {
    pub rollback: Rollback,
    pub bump: u8,
}

impl RollbackAccount {
    pub const SEED_PREFIX: &'static str = "rollback";

    pub const SIZE: usize = 8 + 512 + 1;

    pub fn set(&mut self, rollback: Rollback, bump: u8) {
        self.rollback = rollback;
        self.bump = bump
    }
}

#[account]
pub struct PendingRequest {
    pub sources: Vec<Pubkey>,
}

impl PendingRequest {
    pub const SEED_PREFIX: &'static str = "req";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 320;
}

#[account]
pub struct PendingResponse {
    pub sources: Vec<Pubkey>,
}

impl PendingResponse {
    pub const SEED_PREFIX: &'static str = "res";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 320;
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
    pub bump: u8,
}

impl ProxyRequest {
    pub const SEED_PREFIX: &'static str = "proxy";

    pub const SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + 1024 + 1;

    pub fn set(&mut self, req: CSMessageRequest, bump: u8) {
        self.req = req;
        self.bump = bump
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub to: String,
    pub sn: i64,
    pub msg: Vec<u8>,
}
