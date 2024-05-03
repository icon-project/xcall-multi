use soroban_sdk::{token, xdr::ToXdr, Address, Bytes, Env};

use crate::{
    contract::Xcall,
    errors::ContractError,
    states::{MAX_DATA_SIZE, MAX_ROLLBACK_SIZE},
    types::rollback::Rollback,
};

pub const C_ASCII_VALUE: u32 = 67;
pub const SC_VALUE_START_INDEX: u32 = 8;

impl Xcall {
    pub fn ensure_admin(e: &Env) -> Result<Address, ContractError> {
        let admin = Self::admin(&e)?;
        admin.require_auth();

        Ok(admin)
    }

    pub fn ensure_fee_handler(e: &Env) -> Result<Address, ContractError> {
        let fee_handler = Self::get_fee_handler(&e)?;
        fee_handler.require_auth();

        Ok(fee_handler)
    }

    pub fn ensure_data_size(len: usize) -> Result<(), ContractError> {
        if len > MAX_DATA_SIZE as usize {
            return Err(ContractError::MaxDataSizeExceeded);
        }

        Ok(())
    }

    pub fn ensure_rollback_size(msg: &Bytes) -> Result<(), ContractError> {
        if !msg.is_empty() && msg.len() > MAX_ROLLBACK_SIZE as u32 {
            return Err(ContractError::MaxRollbackSizeExceeded);
        }

        Ok(())
    }

    pub fn ensure_rollback_enabled(rollback: &Rollback) -> Result<(), ContractError> {
        if !rollback.enabled() {
            return Err(ContractError::RollbackNotEnabled);
        }

        Ok(())
    }

    pub fn is_contract(e: &Env, address: &Address) -> bool {
        let bytes = address.to_string().to_xdr(&e);
        let char_index: u32 = bytes.get(SC_VALUE_START_INDEX).unwrap().into();
        if char_index == C_ASCII_VALUE {
            return true;
        }
        false
    }

    pub fn transfer_token(
        e: &Env,
        from: &Address,
        to: &Address,
        amount: &u128,
    ) -> Result<(), ContractError> {
        let config = Self::get_config(&e)?;
        let client = token::Client::new(&e, &config.native_token);

        client.transfer(&from, &to, &(*amount as i128));
        Ok(())
    }
}
