use core::u128;

use soroban_sdk::{Address, Env, String};

use crate::contract::CentralizedConnection;
use crate::errors::ContractError;
use crate::types::StorageKey;

impl CentralizedConnection {
    pub fn is_initialized(e: &Env) -> Result<(), ContractError> {
        let initialized = e.storage().instance().has(&StorageKey::Admin);
        if initialized {
            Err(ContractError::AlreadyInitialized)
        } else {
            Ok(())
        }
    }

    pub fn admin(e: &Env) -> Result<Address, ContractError> {
        if let Some(addr) = e.storage().instance().get(&StorageKey::Admin) {
            Ok(addr)
        } else {
            Err(ContractError::Uninitialized)
        }
    }

    pub fn get_xcall(e: &Env) -> Result<Address, ContractError> {
        if let Some(addr) = e.storage().instance().get(&StorageKey::Xcall) {
            Ok(addr)
        } else {
            Err(ContractError::Uninitialized)
        }
    }

    pub fn native_token(e: &Env) -> Result<Address, ContractError> {
        if let Some(addr) = e.storage().instance().get(&StorageKey::Xlm) {
            Ok(addr)
        } else {
            Err(ContractError::Uninitialized)
        }
    }

    pub fn get_conn_sn(e: &Env) -> Result<u128, ContractError> {
        if let Some(sn) = e.storage().instance().get(&StorageKey::ConnSn) {
            Ok(sn)
        } else {
            Err(ContractError::Uninitialized)
        }
    }

    pub fn get_next_conn_sn(e: &Env) -> u128 {
        let mut sn = e.storage().instance().get(&StorageKey::ConnSn).unwrap_or(0);
        sn += 1;
        sn
    }

    pub fn get_msg_fee(e: &Env, network_id: String) -> u128 {
        e.storage()
            .instance()
            .get(&StorageKey::MessageFee(network_id))
            .unwrap_or(0)
    }

    pub fn get_res_fee(e: &Env, network_id: String) -> u128 {
        e.storage()
            .instance()
            .get(&StorageKey::ResponseFee(network_id))
            .unwrap_or(0)
    }

    pub fn get_receipt(e: &Env, network_id: String, sn: u128) -> bool {
        e.storage()
            .instance()
            .get(&StorageKey::Receipts(network_id, sn))
            .unwrap_or(false)
    }

    pub fn store_receipt(e: &Env, network_id: String, sn: u128) {
        let key = StorageKey::Receipts(network_id, sn);
        e.storage().instance().set(&key, &true);
    }

    pub fn store_admin(e: &Env, admin: Address) {
        e.storage().instance().set(&StorageKey::Admin, &admin);
    }

    pub fn store_xcall(e: &Env, xcall: Address) {
        e.storage().instance().set(&StorageKey::Xcall, &xcall);
    }

    pub fn store_native_token(e: &Env, address: Address) {
        e.storage().instance().set(&StorageKey::Xlm, &address);
    }

    pub fn store_conn_sn(e: &Env, sn: u128) {
        e.storage().instance().set(&StorageKey::ConnSn, &sn);
    }

    pub fn store_network_fee(e: &Env, network_id: String, message_fee: u128, response_fee: u128) {
        let msg_key = StorageKey::MessageFee(network_id.clone());
        let res_key = StorageKey::ResponseFee(network_id.clone());

        e.storage().instance().set(&msg_key, &message_fee);
        e.storage().instance().set(&res_key, &response_fee);
    }
}
