use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Vec};

use crate::{errors::ContractError, event, helpers, storage, types::InitializeMsg};

#[contract]
pub struct ClusterConnection;

#[contractimpl]
impl ClusterConnection {
    pub fn initialize(env: Env, msg: InitializeMsg) -> Result<(), ContractError> {
        storage::is_initialized(&env)?;

        storage::store_native_token(&env, msg.native_token);
        storage::store_conn_sn(&env, 0);
        storage::store_relayer(&env, msg.relayer);
        storage::store_admin(&env, msg.admin);
        storage::store_xcall(&env, msg.xcall_address);
        storage::store_upgrade_authority(&env, msg.upgrade_authority);
        storage::store_validator_threshold(&env, 0);
        storage::store_validators(&env, Vec::new(&env));

        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        let address = storage::admin(&env)?;
        Ok(address)
    }

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

    pub fn set_relayer(env: Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_relayer(&env, address);
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

    pub fn recv_message_with_signatures(
        env: Env,
        src_network: String,
        conn_sn: u128,
        msg: Bytes,
        signatures: Vec<BytesN<65>>,
    ) -> Result<(), ContractError> {
        helpers::ensure_relayer(&env)?;

        if !helpers::verify_signatures(&env, signatures, &src_network, &conn_sn, &msg){
            return Err(ContractError::SignatureVerificationFailed);
        };

        if storage::get_sn_receipt(&env, src_network.clone(), conn_sn) {
            return Err(ContractError::DuplicateMessage);
        }
        storage::store_receipt(&env, src_network.clone(), conn_sn);

        helpers::call_xcall_handle_message(&env, &src_network, msg)?;
        Ok(())
    }

    pub fn set_fee(
        env: Env,
        network_id: String,
        message_fee: u128,
        response_fee: u128,
    ) -> Result<(), ContractError> {
        helpers::ensure_relayer(&env)?;

        storage::store_network_fee(&env, network_id, message_fee, response_fee);
        Ok(())
    }

    pub fn claim_fees(env: Env) -> Result<(), ContractError> {
        let admin = helpers::ensure_relayer(&env)?;

        let token_addr = storage::native_token(&env)?;
        let client = token::Client::new(&env, &token_addr);
        let balance = client.balance(&env.current_contract_address());

        client.transfer(&env.current_contract_address(), &admin, &balance);
        Ok(())
    }

    pub fn update_validators(env: Env, pub_keys: Vec<BytesN<65>>, threshold: u32) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        let mut validators = Vec::new(&env);

        for address in pub_keys.clone() {
            if !validators.contains(&address) {
                validators.push_back(address);
            }
        }
        if (validators.len() as u32) < threshold {
            return Err(ContractError::ThresholdExceeded);
            
        }
        storage::store_validators(&env, pub_keys);
        storage::store_validator_threshold(&env, threshold);
        Ok(())
    }

    pub fn get_validators_threshold(env: Env) -> Result<u32, ContractError> {
        let threshold = storage::get_validators_threshold(&env).unwrap();
        Ok(threshold)
    }

    pub fn set_validators_threshold(env: Env, threshold: u32) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        let validators = storage::get_validators(&env).unwrap();
        if (validators.len() as u32) < threshold {
            return Err(ContractError::ThresholdExceeded);
        }
        storage::store_validator_threshold(&env, threshold);
        Ok(())
    }

    pub fn get_validators(env: Env) -> Result<Vec<BytesN<65>>, ContractError> {
        let validators = storage::get_validators(&env).unwrap();
        Ok(validators)
    }

    pub fn get_relayer(env: Env) -> Result<Address, ContractError> {
        let address = storage::relayer(&env)?;
        Ok(address)
    }

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
