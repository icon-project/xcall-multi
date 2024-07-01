use anchor_lang::prelude::*;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub to: String,
    pub sn: i64,
    pub msg: Vec<u8>,
}
