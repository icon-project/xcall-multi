use anchor_lang::prelude::*;

use crate::{constants::*, error::XcallError};

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
