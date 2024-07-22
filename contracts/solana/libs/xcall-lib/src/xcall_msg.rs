use anchor_lang::prelude::*;

use crate::network_address::NetworkAddress;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessage {
    pub msg: Vec<u8>,
    pub to: NetworkAddress,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleMessage {
    pub from_nid: String,
    pub message: Vec<u8>,
    pub sequence_no: u128,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HandleError {
    pub from_nid: String,
    pub sequence_no: u128,
}
