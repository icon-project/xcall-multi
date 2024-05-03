use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    messages::envelope::Envelope,
    types::{message::InitializeMsg, network_address::NetworkAddress, storage_types::Config},
};

#[contract]
pub struct Xcall;

#[contractimpl]
impl Xcall {
    pub fn initialize(env: Env, msg: InitializeMsg) -> Result<(), ContractError> {
        Self::is_initialized(&env)?;

        Self::store_admin(&env, &msg.sender);
        Self::store_fee_handler(&env, &msg.sender);
        Self::store_config(
            &env,
            Config {
                network_id: msg.network_id,
                native_token: msg.native_token,
            },
        );

        Ok(())
    }

    pub fn set_admin(env: &Env, address: Address) -> Result<(), ContractError> {
        Self::ensure_admin(&env)?;
        Self::store_admin(&env, &address);

        Ok(())
    }

    pub fn set_protocol_fee(env: &Env, fee: u128) -> Result<(), ContractError> {
        Self::ensure_fee_handler(&env)?;
        Self::store_protocol_fee(&env, fee);

        Ok(())
    }

    pub fn set_protocol_fee_handler(env: Env, address: Address) -> Result<(), ContractError> {
        Self::ensure_admin(&env)?;
        Self::store_fee_handler(&env, &address);

        Ok(())
    }

    pub fn set_default_connection(
        env: &Env,
        nid: String,
        address: Address,
    ) -> Result<(), ContractError> {
        Self::ensure_admin(&env)?;
        Self::store_default_connection(&env, nid, &address);

        Ok(())
    }

    pub fn send_call(
        env: Env,
        envelope: Envelope,
        to: NetworkAddress,
        fee: u128,
        sender: Address,
    ) -> Result<(), ContractError> {
        Self::send_message(&env, envelope, to, fee, sender)?;
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        let admin = Self::admin(&env)?;
        Ok(admin)
    }

    pub fn get_network_address(env: Env) -> Result<NetworkAddress, ContractError> {
        let network_address = Self::get_own_network_address(&env)?;
        Ok(network_address)
    }

    pub fn get_fee(
        env: Env,
        nid: String,
        rollback: bool,
        sources: Option<Vec<String>>,
    ) -> Result<u128, ContractError> {
        let fee = Self::get_total_fee(&env, &nid, sources.unwrap_or(Vec::new(&env)), rollback)?;
        Ok(fee)
    }

    pub fn get_protocol_fee(env: &Env) -> Result<u128, ContractError> {
        let fee = Self::protocol_fee(&env);
        Ok(fee)
    }

    pub fn get_protocol_fee_handler(env: Env) -> Result<Address, ContractError> {
        let fee_handler = Self::get_fee_handler(&env)?;
        Ok(fee_handler)
    }

    pub fn get_default_connection(env: Env, nid: String) -> Result<Address, ContractError> {
        let connection = Self::default_connection(&env, nid)?;
        Ok(connection)
    }

    pub fn verify_success(env: Env, sn: u128) -> bool {
        Self::get_successful_response(&env, sn)
    }

    pub fn handle_message(_env: Env, _from_nid: String, _msg: Bytes) -> () {}

    pub fn handle_error(_env: Env, _sn: u128) -> () {}
}
