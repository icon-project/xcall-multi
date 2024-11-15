use soroban_sdk::{Address, BytesN, Env, String};

use crate::{
    error::ContractError,
    types::{storage_types::StorageKey, swap_order::SwapOrder},
};

const DAY_IN_LEDGERS: u32 = 17280; // assumes 5s a ledger

const LEDGER_THRESHOLD_INSTANCE: u32 = DAY_IN_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + DAY_IN_LEDGERS; // ~ 31 days

const LEDGER_THRESHOLD_PERSISTENT: u32 = DAY_IN_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_PERSISTENT: u32 = LEDGER_THRESHOLD_PERSISTENT + DAY_IN_LEDGERS; // ~ 31 days

pub fn is_initialized(e: &Env) -> Result<(), ContractError> {
    let initialized = e.storage().instance().has(&StorageKey::Admin);
    if initialized {
        Err(ContractError::AlreadyInitialized)
    } else {
        Ok(())
    }
}

pub fn admin(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Admin)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_order(e: &Env, id: u128) -> Result<SwapOrder, ContractError> {
    let key = StorageKey::Orders(id);
    e.storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::OrderNotFound)
}

pub fn get_receipt(e: &Env, network_id: String, conn_sn: u128) -> bool {
    let key = StorageKey::Receipts(network_id, conn_sn);
    let is_received = e.storage().persistent().get(&key).unwrap_or(false);
    if is_received {
        extend_persistent(e, &key);
    }

    is_received
}

pub fn get_fee_handler(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::FeeHandler)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_upgrade_authority(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::UpgradeAuthority)
        .ok_or(ContractError::Uninitialized)
}

pub fn protocol_fee(e: &Env) -> u128 {
    e.storage()
        .instance()
        .get(&StorageKey::ProtocolFee)
        .unwrap_or(0)
}

pub fn nid(e: &Env) -> Result<String, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Nid)
        .ok_or(ContractError::Uninitialized)
}

pub fn deposit_id(e: &Env) -> Result<u128, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::DepositId)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_contract_version(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&StorageKey::Version)
        .unwrap_or(1)
}

pub fn set_contract_version(e: &Env, new_version: u32) {
    e.storage()
        .instance()
        .set(&StorageKey::Version, &new_version);
}

pub fn increment_conn_sn(e: &Env) -> u128 {
    let mut sn: u128 = e.storage().instance().get(&StorageKey::ConnSn).unwrap_or(0);
    sn += 1;
    e.storage().instance().set(&StorageKey::ConnSn, &sn);

    sn
}

pub fn increment_deposit_id(e: &Env) -> u128 {
    e.storage()
        .instance()
        .update(&StorageKey::DepositId, |value| -> u128 {
            if let Some(req_id) = value {
                return req_id + 1;
            }
            1
        })
}

pub fn store_finished_order(e: &Env, order_hash: &BytesN<32>) {
    e.storage()
        .persistent()
        .set(&StorageKey::FinishedOrders(order_hash.clone()), &true)
}

pub fn remove_order(e: &Env, id: u128) {
    e.storage().persistent().remove(&StorageKey::Orders(id));
}

pub fn order_finished(e: &Env, order_hash: &BytesN<32>) -> bool {
    e.storage()
        .persistent()
        .get(&StorageKey::FinishedOrders(order_hash.clone()))
        .unwrap_or(false)
}

pub fn store_receipt(e: &Env, network_id: String, conn_sn: u128) {
    let key = StorageKey::Receipts(network_id, conn_sn);
    e.storage().persistent().set(&key, &true);
    extend_persistent(e, &key);
}

pub fn store_admin(e: &Env, address: &Address) {
    e.storage().instance().set(&StorageKey::Admin, &address);
    extend_instance(&e);
}

pub fn store_order(e: &Env, id: u128, order: &SwapOrder) {
    e.storage().persistent().set(&StorageKey::Orders(id), order);
}

pub fn store_network_id(e: &Env, network_id: &String) {
    e.storage().instance().set(&StorageKey::Nid, network_id);
    extend_instance(&e);
}

pub fn store_native_token(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&StorageKey::NativeToken, address);
    extend_instance(&e);
}

pub fn store_fee_handler(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&StorageKey::FeeHandler, &address);
    extend_instance(e)
}

pub fn store_protocol_fee(e: &Env, fee: u128) {
    e.storage().instance().set(&StorageKey::ProtocolFee, &fee);
    extend_instance(e)
}

pub fn store_upgrade_authority(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&StorageKey::UpgradeAuthority, &address);
    extend_instance(e)
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn extend_persistent(e: &Env, key: &StorageKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, LEDGER_THRESHOLD_PERSISTENT, LEDGER_BUMP_PERSISTENT);
}
