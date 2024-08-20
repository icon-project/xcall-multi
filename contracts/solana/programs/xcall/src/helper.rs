use anchor_lang::{
    prelude::*,
    solana_program::{hash, sysvar::instructions::get_instruction_relative},
};
use xcall_lib::{
    xcall_connection_type::CONNECTION_AUTHORITY_SEED, xcall_dapp_type::DAPP_AUTHORITY_SEED,
};

use crate::{constants::*, error::*};

pub fn ensure_data_length(data: &[u8]) -> Result<()> {
    require_gte!(
        MAX_DATA_SIZE,
        data.len() as usize,
        XcallError::MaxDataSizeExceeded
    );

    Ok(())
}

pub fn ensure_rollback_size(rollback: &[u8]) -> Result<()> {
    if rollback.is_empty() {
        return Err(XcallError::NoRollbackData.into());
    }
    if rollback.len() > MAX_ROLLBACK_SIZE {
        return Err(XcallError::MaxRollbackSizeExceeded.into());
    }

    Ok(())
}

pub fn ensure_dapp_authority(dapp_program_id: &Pubkey, dapp_authority_key: Pubkey) -> Result<()> {
    let (derived_key, _) =
        Pubkey::find_program_address(&[DAPP_AUTHORITY_SEED.as_bytes()], &dapp_program_id);
    if derived_key != dapp_authority_key {
        return Err(XcallError::InvalidSigner.into());
    }

    Ok(())
}

pub fn ensure_connection_authority(
    conn_program_id: &Pubkey,
    conn_authority_key: Pubkey,
) -> Result<()> {
    let (derived_key, _) =
        Pubkey::find_program_address(&[CONNECTION_AUTHORITY_SEED.as_bytes()], &conn_program_id);
    if derived_key != conn_authority_key {
        return Err(XcallError::InvalidSigner.into());
    }

    Ok(())
}

pub fn is_program(sysvar_account_info: &AccountInfo) -> Result<bool> {
    let current_ix = get_instruction_relative(0, sysvar_account_info)?;
    if current_ix.program_id != crate::id() {
        return Ok(true);
    }

    Ok(false)
}

pub fn hash_data(data: &Vec<u8>) -> Vec<u8> {
    return hash::hash(data).to_bytes().to_vec();
}

pub fn get_instruction_data(ix_name: &str, data: Vec<u8>) -> Vec<u8> {
    let preimage = format!("{}:{}", "global", ix_name);

    let mut ix_discriminator = [0u8; 8];
    ix_discriminator.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);

    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&ix_discriminator);
    ix_data.extend_from_slice(&data);

    ix_data
}
