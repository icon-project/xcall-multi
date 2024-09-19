use soroban_sdk::{Address, Env, String};

use crate::{
    errors::ContractError,
    types::{NetworkFee, StorageKey},
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

pub fn get_upgrade_authority(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::UpgradeAuthority)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_xcall(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Xcall)
        .ok_or(ContractError::Uninitialized)
}

pub fn native_token(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Xlm)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_conn_sn(e: &Env) -> Result<u128, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::ConnSn)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_next_conn_sn(e: &Env) -> u128 {
    let mut sn = e.storage().instance().get(&StorageKey::ConnSn).unwrap_or(0);
    sn += 1;
    sn
}

pub fn get_msg_fee(e: &Env, network_id: String) -> Result<u128, ContractError> {
    let key = StorageKey::NetworkFee(network_id);
    let network_fee: NetworkFee = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::NetworkNotSupported)?;
    extend_persistent(e, &key);

    Ok(network_fee.message_fee)
}

pub fn get_res_fee(e: &Env, network_id: String) -> Result<u128, ContractError> {
    let key = StorageKey::NetworkFee(network_id);
    let network_fee: NetworkFee = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::NetworkNotSupported)?;
    extend_persistent(e, &key);

    Ok(network_fee.response_fee)
}

pub fn get_sn_receipt(e: &Env, network_id: String, sn: u128) -> bool {
    let key = StorageKey::Receipts(network_id, sn);
    let is_received = e.storage().persistent().get(&key).unwrap_or(false);
    if is_received {
        extend_persistent(e, &key);
    }

    is_received
}

pub fn store_receipt(e: &Env, network_id: String, sn: u128) {
    let key = StorageKey::Receipts(network_id, sn);
    e.storage().persistent().set(&key, &true);
    extend_persistent(e, &key);
}

pub fn store_admin(e: &Env, admin: Address) {
    e.storage().instance().set(&StorageKey::Admin, &admin);
}

pub fn store_upgrade_authority(e: &Env, address: Address) {
    e.storage()
        .instance()
        .set(&StorageKey::UpgradeAuthority, &address);
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
    let key = StorageKey::NetworkFee(network_id);
    let network_fee = NetworkFee {
        message_fee,
        response_fee,
    };
    e.storage().persistent().set(&key, &network_fee);
    extend_persistent(e, &key);
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
