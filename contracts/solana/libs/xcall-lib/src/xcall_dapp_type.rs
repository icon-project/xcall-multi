use anchor_lang::prelude::*;

use crate::network_address::NetworkAddress;

pub const HANDLE_CALL_MESSAGE_IX: &str = "handle_call_message";

pub const QUERY_HANDLE_CALL_MESSAGE_IX: &str = "query_handle_call_message_accounts";

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleCallMessageArgs {
    pub from: NetworkAddress,
    pub data: Vec<u8>,
    pub protocols: Vec<String>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleCallMessageResponse {
    pub success: bool,
    pub message: String,
}
