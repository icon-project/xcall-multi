use anchor_lang::prelude::*;
use xcall_lib::xcall_dapp_type;

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

#[account]
#[derive(Debug)]
pub struct Connection {
    pub src_endpoint: String,
    pub dst_endpoint: String,
}

#[account]
pub struct Authority {
    pub bump: u8,
}

impl Authority {
    pub const SEED_PREFIX: &'static str = xcall_dapp_type::DAPP_AUTHORITY_SEED;
    pub const MAX_SPACE: usize = 8 + 1;
}
