use soroban_sdk::{Address, Bytes, Env, String, Vec};

use crate::errors::ContractError;
use crate::types::{Connection, StorageKey};

pub fn is_initialized(e: &Env) -> Result<(), ContractError> {
    let initialized = e.storage().instance().has(&StorageKey::XcallAddress);
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

pub fn native_token(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Xlm)
        .ok_or(ContractError::Uninitialized)
}

pub fn store_admin(e: &Env, admin: Address) {
    e.storage().instance().set(&StorageKey::Admin, &admin);
}

pub fn store_native_token(e: &Env, address: Address) {
    e.storage().instance().set(&StorageKey::Xlm, &address);
}

pub fn store_xcall_address(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&StorageKey::XcallAddress, address);
}

pub fn store_sn_no(e: &Env, sn: &u128) {
    e.storage().instance().set(&StorageKey::Sn, sn)
}

pub fn store_rollback(e: &Env, sn: &u128, bytes: &Bytes) {
    e.storage()
        .instance()
        .set(&StorageKey::Rollback(*sn), bytes)
}

pub fn add_new_connection(e: &Env, network_id: String, conn: Connection) {
    let mut connections: Vec<Connection> = e
        .storage()
        .instance()
        .get(&StorageKey::Connections(network_id.clone()))
        .unwrap_or(Vec::new(&e));

    connections.push_back(conn);
    e.storage()
        .instance()
        .set(&StorageKey::Connections(network_id), &connections);
}

pub fn get_connections(e: &Env, network_id: String) -> Result<Vec<Connection>, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Connections(network_id))
        .ok_or(ContractError::ConnectionNotFound)
}

pub fn get_xcall_address(e: &Env) -> Result<Address, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::XcallAddress)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_sn(e: &Env) -> Result<u128, ContractError> {
    e.storage()
        .instance()
        .get(&StorageKey::Sn)
        .ok_or(ContractError::Uninitialized)
}

pub fn get_next_sn(e: &Env) -> Result<u128, ContractError> {
    let mut sn = get_sn(&e)?;
    sn += 1;
    e.storage().instance().set(&StorageKey::Sn, &sn);

    Ok(sn)
}
