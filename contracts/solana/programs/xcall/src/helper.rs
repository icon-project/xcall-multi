use anchor_lang::{prelude::*, solana_program::hash};

use crate::{constants::*, error::*};

pub fn ensure_data_length(data: &[u8]) -> Result<()> {
    require_gte!(
        MAX_DATA_SIZE,
        data.len() as usize,
        XcallError::MaxDataSizeExceeded
    );

    Ok(())
}

pub fn ensure_rollback_length(rollback: &[u8]) -> Result<()> {
    require_gte!(
        MAX_ROLLBACK_SIZE,
        rollback.len() as usize,
        XcallError::MaxRollbackSizeExceeded
    );

    Ok(())
}

pub fn ensure_program(account: &AccountInfo) -> Result<()> {
    require_eq!(account.executable, true, XcallError::RollbackNotPossible);

    Ok(())
}

pub fn get_instruction_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", "global", name);

    let mut ix_discriminator = [0u8; 8];
    ix_discriminator.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);

    ix_discriminator
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
