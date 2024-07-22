use anchor_lang::prelude::*;

use crate::network_address::NetworkAddress;

pub const HANDLE_CALL_MESSAGE_IX: &str = "handle_call_message";

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleCallMessage {
    pub from: NetworkAddress,
    pub data: Vec<u8>,
    pub protocols: Option<Vec<String>>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleCallMessageResponse {
    pub success: bool,
    pub message: String,
}
