use anchor_lang::prelude::borsh;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct CpiDappResponse {
    pub success: bool,
    pub data: Option<String>,
}
