use anchor_lang::prelude::*;

use crate::network_address::NetworkAddress;

pub const SEND_CALL_IX: &str = "send_call";
pub const HANDLE_MESSAGE_IX: &str = "handle_message";
pub const HANDLE_ERROR_IX: &str = "handle_error";
pub const EXECUTE_CALL_IX: &str = "execute_call";

pub const QUERY_HANDLE_MESSAGE_ACCOUNTS_IX: &str = "query_handle_message_accounts";
pub const QUERY_HANDLE_ERROR_ACCOUNTS_IX: &str = "query_handle_error_accounts";
pub const QUERY_EXECUTE_CALL_ACCOUNTS_IX: &str = "query_execute_call_accounts";

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendCallArgs {
    pub msg: Vec<u8>,
    pub to: NetworkAddress,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleMessageArgs {
    pub from_nid: String,
    pub message: Vec<u8>,
    pub sequence_no: u128,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleErrorArgs {
    pub sequence_no: u128,
}
