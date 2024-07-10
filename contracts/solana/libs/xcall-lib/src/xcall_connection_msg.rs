use anchor_lang::prelude::*;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessage {
    pub to: String,
    pub sn: i64,
    pub msg: Vec<u8>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct GetFee {
    pub network_id: String,
    pub response: bool,
}
