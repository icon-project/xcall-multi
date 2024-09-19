use soroban_sdk::{Address, BytesN, Env, String, Vec};
use soroban_xcall_lib::network_address::NetworkAddress;

use crate::{
    errors::ContractError,
    types::{
        request::CSMessageRequest,
        rollback::Rollback,
        storage_types::{Config, StorageKey},
    },
};

const DAY_IN_LEDGERS: u32 = 17280; // assumes 5s a ledger

const LEDGER_THRESHOLD_INSTANCE: u32 = DAY_IN_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + DAY_IN_LEDGERS; // ~ 31 days

const LEDGER_THRESHOLD_PERSISTENT: u32 = DAY_IN_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_PERSISTENT: u32 = LEDGER_THRESHOLD_PERSISTENT + DAY_IN_LEDGERS; // ~ 31 days

const LEDGER_THRESHOLD_REQUEST: u32 = DAY_IN_LEDGERS * 7; // ~ 7 days
const LEDGER_BUMP_REQUEST: u32 = LEDGER_THRESHOLD_REQUEST + DAY_IN_LEDGERS; // ~ 8 days

pub const MAX_ROLLBACK_SIZE: u64 = 1024;
pub const MAX_DATA_SIZE: u64 = 2048;

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

pub fn get_config(e: &Env) -> Result<Config, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Config)
        .ok_or(ContractError::Uninitialized)
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

pub fn default_connection(e: &Env, nid: String) -> Result<Address, ContractError> {
    let key = StorageKey::DefaultConnections(nid);
    let connection = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::NoDefaultConnection)?;
    extend_persistent(e, &key);

    connection
}

pub fn get_rollback(e: &Env, sequence_no: u128) -> Result<Rollback, ContractError> {
    let key = StorageKey::Rollback(sequence_no);
    let rollback = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::CallRequestNotFound)?;
    extend_persistent_request(e, &key);

    rollback
}

pub fn get_successful_response(e: &Env, sn: u128) -> bool {
    let key = StorageKey::SuccessfulResponses(sn);
    let res = e.storage().persistent().get(&key).unwrap_or(false);
    if res {
        extend_persistent(e, &key)
    }

    res
}

pub fn get_next_sn(e: &Env) -> u128 {
    let mut sn: u128 = e.storage().instance().get(&StorageKey::Sn).unwrap_or(0);
    sn += 1;
    e.storage().instance().set(&StorageKey::Sn, &sn);

    sn
}

pub fn get_proxy_request(e: &Env, req_id: u128) -> Result<CSMessageRequest, ContractError> {
    let key = StorageKey::ProxyRequest(req_id);
    let request = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::InvalidRequestId)?;
    extend_persistent_request(e, &key);

    request
}

pub fn get_pending_request(e: &Env, hash: BytesN<32>) -> Vec<String> {
    let key = StorageKey::PendingRequests(hash);
    let pending_request = e.storage().persistent().get(&key).unwrap_or(Vec::new(&e));
    if pending_request.len() > 0 {
        extend_persistent_request(e, &key);
    }

    pending_request
}

pub fn get_pending_response(e: &Env, hash: BytesN<32>) -> Vec<String> {
    let key = StorageKey::PendingResponses(hash);
    let pending_response = e.storage().persistent().get(&key).unwrap_or(Vec::new(&e));
    if pending_response.len() > 0 {
        extend_persistent_request(e, &key);
    }

    pending_response
}

pub fn get_own_network_address(e: &Env) -> Result<NetworkAddress, ContractError> {
    let config = get_config(&e)?;
    let from = NetworkAddress::new(
        &e,
        config.network_id,
        e.current_contract_address().to_string(),
    );

    Ok(from)
}

pub fn store_admin(e: &Env, address: &Address) {
    e.storage().instance().set(&StorageKey::Admin, &address);
    extend_instance(&e);
}

pub fn store_config(e: &Env, config: Config) {
    e.storage().instance().set(&StorageKey::Config, &config)
}

pub fn store_fee_handler(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&StorageKey::FeeHandler, &address);
    extend_instance(e)
}

pub fn store_upgrade_authority(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&StorageKey::UpgradeAuthority, &address);
    extend_instance(e)
}

pub fn store_protocol_fee(e: &Env, fee: u128) {
    e.storage().instance().set(&StorageKey::ProtocolFee, &fee);
    extend_instance(e)
}

pub fn store_default_connection(e: &Env, nid: String, address: &Address) {
    let key = StorageKey::DefaultConnections(nid);
    e.storage().persistent().set(&key, &address);
    extend_persistent(e, &key);
}

pub fn store_rollback(e: &Env, sn: u128, rollback: &Rollback) {
    let key = StorageKey::Rollback(sn);
    e.storage().persistent().set(&key, rollback);
    extend_persistent_request(e, &key)
}

pub fn remove_rollback(e: &Env, sn: u128) {
    e.storage().persistent().remove(&StorageKey::Rollback(sn));
}

pub fn store_proxy_request(e: &Env, req_id: u128, request: &CSMessageRequest) {
    let key = StorageKey::ProxyRequest(req_id);
    e.storage().persistent().set(&key, request);
    extend_persistent_request(e, &key)
}

pub fn remove_proxy_request(e: &Env, req_id: u128) {
    e.storage()
        .persistent()
        .remove(&StorageKey::ProxyRequest(req_id))
}

pub fn store_pending_request(e: &Env, hash: BytesN<32>, sources: &Vec<String>) {
    let key = StorageKey::PendingRequests(hash.clone());
    e.storage().persistent().set(&key, sources);
    extend_persistent_request(e, &key)
}

pub fn remove_pending_request(e: &Env, hash: BytesN<32>) {
    e.storage()
        .persistent()
        .remove(&StorageKey::PendingRequests(hash))
}

pub fn store_pending_response(e: &Env, hash: BytesN<32>, sources: &Vec<String>) {
    let key = StorageKey::PendingResponses(hash);
    e.storage().persistent().set(&key, sources);
    extend_persistent_request(e, &key)
}

pub fn remove_pending_response(e: &Env, hash: BytesN<32>) {
    e.storage()
        .persistent()
        .remove(&StorageKey::PendingResponses(hash))
}

pub fn increment_last_request_id(e: &Env) -> u128 {
    e.storage()
        .instance()
        .update(&StorageKey::LastReqId, |value| -> u128 {
            if let Some(req_id) = value {
                return req_id + 1;
            }
            1
        })
}

pub fn save_success_response(e: &Env, sn: u128) {
    let key = StorageKey::SuccessfulResponses(sn);
    e.storage().persistent().set(&key, &true);
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

pub fn extend_persistent_request(e: &Env, key: &StorageKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, LEDGER_THRESHOLD_REQUEST, LEDGER_BUMP_REQUEST);
}
