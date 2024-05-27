use anchor_lang::prelude::borsh::BorshDeserialize;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::{prelude::borsh::BorshSerialize, solana_program::account_info::AccountInfo};

use crate::{CSMessageRequest, NetworkAddress};

const DISCRIMINANT_END: usize = 8;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FeesState {
    pub message_fees: u64,
    pub response_fees: u64,
}

pub fn transfer_lamports(source: &AccountInfo, destination: &AccountInfo) -> Result<()> {
    // Transfer all lamports from source to destination
    let source_lamports = source.get_lamports();
    **destination.try_borrow_mut_lamports()? += source_lamports;
    **source.lamports.borrow_mut() = 0;

    // **destination.try_borrow_mut_lamports()? += **source.lamports.borrow();
    // **source.lamports.borrow_mut() = 0;
    Ok(())
}

pub fn get_fee(connection_fee: &AccountInfo, sn: i128) -> Result<u64> {
    if sn < 0 {
        return Ok(0);
    }
    let serialized_fee = &connection_fee.try_borrow_mut_data()?[DISCRIMINANT_END..];
    let conn_fee = FeesState::try_from_slice(&serialized_fee).unwrap();
    if sn > 0 {
        return Ok(conn_fee.message_fees + conn_fee.response_fees);
    }
    return Ok(conn_fee.message_fees);
}

pub fn is_reply(reply_data: CSMessageRequest, dst_net: String, sources: Vec<String>) -> bool {
    if reply_data.msg_type != u8::MAX {
        return NetworkAddress::split(reply_data.from).net == dst_net
            && protocol_equals(reply_data.protocols, sources);
    }
    false
}

pub fn protocol_equals(a: Vec<String>, b: Vec<String>) -> bool {
    a.len() == b.len() && a.iter().all(|item| b.contains(item))
}

pub fn sighash(namespace: &str, name: &str) -> Vec<u8> {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash.to_vec()
}
