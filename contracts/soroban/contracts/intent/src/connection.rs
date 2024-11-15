use soroban_sdk::{Address, Bytes, Env, String};

use crate::{error::ContractError, event, helpers, interfaces::*, storage};

pub struct GeneralizedConnection {}

impl IGeneralizedConnection for GeneralizedConnection {
    fn send_message(env: &Env, to: String, msg: Bytes) {
        let conn_sn = storage::increment_conn_sn(&env);

        event::send_message(&env, to, conn_sn, msg);
    }

    fn recv_message(env: &Env, src_network: String, conn_sn: u128) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;

        if storage::get_receipt(&env, src_network.clone(), conn_sn) {
            return Err(ContractError::DuplicateMessage);
        }
        storage::store_receipt(&env, src_network, conn_sn);

        Ok(())
    }

    fn set_admin(env: &Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_admin(&env, &address);

        Ok(())
    }

    fn admin(env: &Env) -> Result<Address, ContractError> {
        storage::admin(&env)
    }

    fn get_receipt(env: &Env, network_id: String, conn_sn: u128) -> bool {
        storage::get_receipt(&env, network_id, conn_sn)
    }
}
