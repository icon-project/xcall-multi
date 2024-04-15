use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};

extern crate alloc;

use crate::errors::ContractError;
use crate::event;
use crate::types::InitializeMsg;

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
        Self::is_initialized(&env)?;

        Self::store_native_token(&env, msg.native_token);
        Self::store_conn_sn(&env, 0);
        Self::store_admin(&env, msg.relayer);
        Self::store_xcall(&env, msg.xcall_address);

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
        let address = Self::admin(&env)?;
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
        Self::ensure_admin(&env)?;
        Self::store_admin(&env, address);
        Ok(())
    }

    pub fn send_message(
        env: Env,
        amount: u128,
        to: String,
        sn: i64,
        msg: Vec<u32>,
    ) -> Result<(), ContractError> {
        // TODO:
        // 1. convert `msg` type to Vec<u8>
        // 2. encode msg to hexadecimal format

        let xcall = Self::ensure_xcall(&env)?;

        let next_conn_sn = Self::get_next_conn_sn(&env);
        Self::store_conn_sn(&env, next_conn_sn);

        let mut fee: u128 = 0;
        if sn >= 0 {
            fee = Self::get_network_fee(&env, to.clone(), sn > 0);
        }

        if fee > amount {
            return Err(ContractError::InsufficientFund);
        }

        Self::transfer_token(&env, &xcall, &env.current_contract_address(), &amount)?;
        event::send_message(&env, to, next_conn_sn, msg);

        Ok(())
    }

    pub fn recv_message(
        env: Env,
        src_network: String,
        conn_sn: u128,
        msg: Vec<u32>,
    ) -> Result<(), ContractError> {
        // TODO:
        // 1. convert `msg` type to String
        // 2. decode msg from hexadecimal format to Vec<u8>

        Self::ensure_admin(&env)?;

        if Self::get_receipt(&env, src_network.clone(), conn_sn) {
            return Err(ContractError::DuplicateMessage);
        }
        Self::store_receipt(&env, src_network.clone(), conn_sn);

        Self::call_xcall_handle_message(&env, &src_network, msg)?;
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
        let address = Self::ensure_admin(&env)?;

        Self::call_xcall_handle_error(&env, address, sn)?;
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
        Self::ensure_admin(&env)?;

        Self::store_network_fee(&env, network_id, message_fee, response_fee);
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
        let admin = Self::ensure_admin(&env)?;

        let token_addr = Self::native_token(&env)?;
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
    pub fn get_fee(env: Env, network_id: String, response: bool) -> u128 {
        Self::get_network_fee(&env, network_id, response)
    }
}
