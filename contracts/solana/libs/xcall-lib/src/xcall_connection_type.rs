use anchor_lang::prelude::*;

pub const GET_FEE_IX: &str = "get_fee";
pub const SEND_MESSAGE_IX: &str = "send_message";

pub const QUERY_SEND_MESSAGE_ACCOUNTS_IX: &str = "query_send_message_accounts";

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub to: String,
    pub sn: i64,
    pub msg: Vec<u8>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct GetFeeArgs {
    pub network_id: String,
    pub response: bool,
}
