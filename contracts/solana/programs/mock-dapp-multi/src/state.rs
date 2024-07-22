use anchor_lang::prelude::*;
use xcall_lib::network_address::NetworkAddress;

#[account]
pub struct Config {
    pub sn: u128,
    pub xcall_address: Pubkey,
    pub bump: u8,
}

impl Config {
    pub const SEED_PREFIX: &'static str = "config";
    pub const MAX_SPACE: usize = 8 + 16 + 32 + 1;
}

#[account]
pub struct Connections {
    pub connections: Vec<Connection>,
}

impl Connections {
    pub const SEED_PREFIX: &'static str = "connections";
    pub const MAX_SPACE: usize = 4 + 256 + 4 + 256;
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub msg: Vec<u8>,
    pub to: NetworkAddress,
}

#[account]
#[derive(Debug)]
pub struct Connection {
    pub src_endpoint: String,
    pub dst_endpoint: String,
}

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
