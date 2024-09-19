use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String};

use crate::{errors::ContractError, event, helpers, storage, types::InitializeMsg};

#[contract]
pub struct CentralizedConnection;

#[contractimpl]
impl CentralizedConnection {
    /// It checks if contract is not already initialized and initialize the contract otherwise
    /// returns the ContractError
    ///
    /// Arguments:
    ///
    /// * `env`: The `Env` type provides access to the environment the contract is executing within
    /// * `msg`: `msg` is a struct which includes admin, xcall and native token address to initialize
    /// the contract
    ///
    /// Returns:
    ///
    /// a `Result<(), ContractError>`
    pub fn initialize(env: Env, msg: InitializeMsg) -> Result<(), ContractError> {
        storage::is_initialized(&env)?;

        storage::store_native_token(&env, msg.native_token);
        storage::store_conn_sn(&env, 0);
        storage::store_admin(&env, msg.relayer);
        storage::store_xcall(&env, msg.xcall_address);
        storage::store_upgrade_authority(&env, msg.upgrade_authority);

        Ok(())
    }

    /// It quries the admin from the contract and returns ContractError if the contract is
    /// not initialized
    ///
    /// Arguments:
    ///
    /// * `env`: The `Env` type provides access to the environment the contract is executing within
    ///
    /// Returns:
    ///
    /// a `Result<Address, ContractError>`
    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        let address = storage::admin(&env)?;
        Ok(address)
    }

    /// It ensures if the caller is an admin and sets the new admin for the contract
    ///
    /// Arguments:
    ///
    /// * `env`: The `Env` type provides access to the environment the contract is executing within
    /// * `address`: The new admin address
    ///
    /// Returns:
    ///
    /// a `Result<(), ContractError>`
    pub fn set_admin(env: Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_admin(&env, address);
        Ok(())
    }

    pub fn get_upgrade_authority(env: Env) -> Result<Address, ContractError> {
        let address = storage::get_upgrade_authority(&env)?;
        Ok(address)
    }

    pub fn set_upgrade_authority(env: &Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_upgrade_authority(&env)?;
        storage::store_upgrade_authority(&env, address);

        Ok(())
    }

    pub fn send_message(
        env: Env,
        tx_origin: Address,
        to: String,
        sn: i64,
        msg: Bytes,
    ) -> Result<(), ContractError> {
        helpers::ensure_xcall(&env)?;

        let next_conn_sn = storage::get_next_conn_sn(&env);
        storage::store_conn_sn(&env, next_conn_sn);

        let mut fee: u128 = 0;
        if sn >= 0 {
            fee = helpers::get_network_fee(&env, to.clone(), sn > 0)?;
        }
        if fee > 0 {
            helpers::transfer_token(&env, &tx_origin, &env.current_contract_address(), &fee)?;
        }
        event::send_message(&env, to, next_conn_sn, msg);

        Ok(())
    }

    pub fn recv_message(
        env: Env,
        src_network: String,
        conn_sn: u128,
        msg: Bytes,
    ) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;

        if storage::get_sn_receipt(&env, src_network.clone(), conn_sn) {
            return Err(ContractError::DuplicateMessage);
        }
        storage::store_receipt(&env, src_network.clone(), conn_sn);

        helpers::call_xcall_handle_message(&env, &src_network, msg)?;
        Ok(())
    }

    /// The function receives message sequence `sn` of failed message from the destination chain
    /// and call xcall to handle the response of failure message
    ///
    /// Arguments:
    ///
    /// * `env`: The `Env` type provides access to the environment the contract is executing within
    /// * `sn`: `sn` is the unique ID of the message send from source chain to the destination chain
    ///
    /// Returns:
    ///
    /// a `Result<(), ContractError)`
    pub fn revert_message(env: &Env, sn: u128) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        helpers::call_xcall_handle_error(&env, sn)?;

        Ok(())
    }

    /// It sets fee required to send and recieve message from source chain to destination chain
    ///
    /// Arguments:
    /// * `env`: The `Env` type provides access to the environment the contract is executing within
    /// * `network_id`: `network_id` is the unique identifier of a blockchain network
    /// * `message_fee`: `message_fee` is the amount of XLM Asset required to send message from
    /// source chain to the destination chain
    /// * `response_fee`: `response_fee` is the amount of XLM Asset required to receive response of
    /// send message from the destination chain
    ///
    /// Returns:
    ///
    /// a `Result<(), ContractError>`
    pub fn set_fee(
        env: Env,
        network_id: String,
        message_fee: u128,
        response_fee: u128,
    ) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;

        storage::store_network_fee(&env, network_id, message_fee, response_fee);
        Ok(())
    }

    /// This function allows admin to claim all the native (XLM) token stored in the contract. First
    /// it checks if the caller is an admin, then it sends `transfer` message to Stellar Asset
    /// Contract (SAC) of native XLM asset and transfers all the contract fund to admin address
    ///
    /// Returns:
    ///
    /// a `Result<(), ContractError>`
    pub fn claim_fees(env: Env) -> Result<(), ContractError> {
        let admin = helpers::ensure_admin(&env)?;

        let token_addr = storage::native_token(&env)?;
        let client = token::Client::new(&env, &token_addr);
        let balance = client.balance(&env.current_contract_address());

        client.transfer(&env.current_contract_address(), &admin, &balance);
        Ok(())
    }

    /// It returns the fee required to send the message for specific blockchain network
    ///
    /// Arguments:
    ///
    /// * `env`: The `Env` type provides access to the environment the contract is executing within.
    /// * `network_id`: `network_id` is the unique blockchain network id stored in the contract storage
    /// * `response`: `response` is a boolean value which indicates if the response fee should be
    /// included or not.
    ///
    /// Returns:
    ///
    /// a `u128` fee required to send message
    pub fn get_fee(env: Env, network_id: String, response: bool) -> Result<u128, ContractError> {
        helpers::get_network_fee(&env, network_id, response)
    }

    pub fn get_receipt(env: Env, network_id: String, sn: u128) -> bool {
        storage::get_sn_receipt(&env, network_id, sn)
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
