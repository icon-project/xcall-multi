use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};
use soroban_xcall_lib::messages::envelope::Envelope;

use crate::{
    errors::ContractError,
    execute_call, handle_message, helpers, send_message, storage,
    types::{message::InitializeMsg, storage_types::Config},
};

#[contract]
pub struct Xcall;

#[contractimpl]
impl Xcall {
    pub fn initialize(env: Env, msg: InitializeMsg) -> Result<(), ContractError> {
        storage::is_initialized(&env)?;

        storage::store_admin(&env, &msg.sender);
        storage::store_fee_handler(&env, &msg.sender);
        storage::store_upgrade_authority(&env, &msg.upgrade_authority);
        storage::store_config(
            &env,
            Config {
                network_id: msg.network_id,
                native_token: msg.native_token,
            },
        );

        Ok(())
    }

    pub fn set_admin(env: &Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_admin(&env, &address);

        Ok(())
    }

    pub fn set_upgrade_authority(env: &Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_upgrade_authority(&env)?;
        storage::store_upgrade_authority(&env, &address);

        Ok(())
    }

    pub fn set_protocol_fee(env: &Env, fee: u128) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_protocol_fee(&env, fee);

        Ok(())
    }

    pub fn set_protocol_fee_handler(env: Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_fee_handler(&env, &address);

        Ok(())
    }

    pub fn set_default_connection(
        env: &Env,
        nid: String,
        address: Address,
    ) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_default_connection(&env, nid, &address);

        Ok(())
    }

    pub fn send_call(
        env: Env,
        tx_origin: Address,
        sender: Address,
        envelope: Envelope,
        to: String,
    ) -> Result<u128, ContractError> {
        send_message::send_call(&env, tx_origin, sender, envelope, to)
    }

    pub fn handle_message(
        env: Env,
        sender: Address,
        from_nid: String,
        msg: Bytes,
    ) -> Result<(), ContractError> {
        handle_message::handle_message(&env, &sender, from_nid, msg)
    }

    pub fn handle_error(env: Env, sender: Address, sequence_no: u128) -> Result<(), ContractError> {
        handle_message::handle_error(&env, sender, sequence_no)
    }

    pub fn execute_call(
        env: Env,
        sender: Address,
        req_id: u128,
        data: Bytes,
    ) -> Result<(), ContractError> {
        execute_call::execute_message(&env, sender, req_id, data)
    }

    pub fn execute_rollback(env: Env, sequence_no: u128) -> Result<(), ContractError> {
        execute_call::execute_rollback_message(&env, sequence_no)
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        let admin = storage::admin(&env)?;
        Ok(admin)
    }

    pub fn get_upgrade_authority(env: Env) -> Result<Address, ContractError> {
        let address = storage::get_upgrade_authority(&env)?;
        Ok(address)
    }

    pub fn get_network_address(env: Env) -> Result<String, ContractError> {
        let network_address = storage::get_own_network_address(&env)?;
        Ok(network_address.to_string())
    }

    pub fn get_fee(
        env: Env,
        nid: String,
        rollback: bool,
        sources: Option<Vec<String>>,
    ) -> Result<u128, ContractError> {
        let fee =
            send_message::get_total_fee(&env, &nid, sources.unwrap_or(Vec::new(&env)), rollback)?;
        Ok(fee)
    }

    pub fn get_protocol_fee(env: &Env) -> Result<u128, ContractError> {
        let fee = storage::protocol_fee(&env);
        Ok(fee)
    }

    pub fn get_protocol_fee_handler(env: Env) -> Result<Address, ContractError> {
        let fee_handler = storage::get_fee_handler(&env)?;
        Ok(fee_handler)
    }

    pub fn get_default_connection(env: Env, nid: String) -> Result<Address, ContractError> {
        let connection = storage::default_connection(&env, nid)?;
        Ok(connection)
    }

    pub fn verify_success(env: Env, sn: u128) -> bool {
        storage::get_successful_response(&env, sn)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        helpers::ensure_upgrade_authority(&env)?;
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    pub fn extend_instance_storage(env: Env) -> Result<(), ContractError> {
        storage::extend_instance(&env);
        Ok(())
    }
}
