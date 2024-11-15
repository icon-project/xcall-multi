use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String};

use crate::{
    cancel,
    connection::GeneralizedConnection,
    error::ContractError,
    fill, helpers,
    interfaces::{IGeneralizedConnection, IIntent},
    storage, swap,
    types::*,
};

#[contract]
pub struct Intent;

#[contractimpl]
impl IIntent for Intent {
    fn initialize(
        env: Env,
        network_id: String,
        admin: Address,
        fee_handler: Address,
        native_token: Address,
        upgrade_authority: Address,
    ) -> Result<(), ContractError> {
        storage::is_initialized(&env)?;

        storage::store_admin(&env, &admin);
        storage::store_native_token(&env, &native_token);
        storage::store_network_id(&env, &network_id);
        storage::store_fee_handler(&env, &fee_handler);
        storage::store_upgrade_authority(&env, &upgrade_authority);

        Ok(())
    }

    fn swap(env: Env, order: SwapOrder) -> Result<(), ContractError> {
        swap::swap_order(&env, order)
    }

    fn fill(
        env: Env,
        order: SwapOrder,
        sender: Address,
        solver_address: String,
    ) -> Result<(), ContractError> {
        fill::fill_order(&env, order, sender, solver_address)
    }

    fn cancel(env: Env, id: u128) -> Result<(), ContractError> {
        cancel::cancel_order(&env, id)
    }

    fn recv_message(
        env: Env,
        src_network: String,
        conn_sn: u128,
        msg: Bytes,
    ) -> Result<(), ContractError> {
        GeneralizedConnection::recv_message(&env, src_network.clone(), conn_sn)?;

        let msg = OrderMessage::decode(&env, msg);
        match msg.message_type() {
            MessageType::FILL => {
                let fill = OrderFill::decode(&env, msg.message());
                fill::resolve_fill(&env, src_network, fill)
            }
            MessageType::CANCEL => {
                let cancel = Cancel::decode(&env, msg.message());
                cancel::resolve_cancel(&env, src_network, cancel.order_bytes())
            }
        }
    }

    fn set_admin(env: Env, address: Address) -> Result<(), ContractError> {
        GeneralizedConnection::set_admin(&env, address)
    }

    fn set_fee_handler(env: Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_fee_handler(&env, &address);

        Ok(())
    }

    fn set_protocol_fee(env: Env, fee: u128) -> Result<(), ContractError> {
        helpers::ensure_admin(&env)?;
        storage::store_protocol_fee(&env, fee);

        Ok(())
    }

    fn set_upgrade_authority(env: &Env, address: Address) -> Result<(), ContractError> {
        helpers::ensure_upgrade_authority(&env)?;
        storage::store_upgrade_authority(&env, &address);

        Ok(())
    }

    fn get_order(env: Env, id: u128) -> Result<SwapOrder, ContractError> {
        storage::get_order(&env, id)
    }

    fn get_finished_order(env: Env, bytes: BytesN<32>) -> Result<bool, ContractError> {
        let order_finished = storage::order_finished(&env, &bytes);

        Ok(order_finished)
    }

    fn get_receipt(env: Env, network_id: String, conn_sn: u128) -> bool {
        GeneralizedConnection::get_receipt(&env, network_id, conn_sn)
    }

    fn get_admin(env: Env) -> Result<Address, ContractError> {
        GeneralizedConnection::admin(&env)
    }

    fn get_upgrade_authority(env: Env) -> Result<Address, ContractError> {
        storage::get_upgrade_authority(&env)
    }

    fn get_nid(env: Env) -> Result<String, ContractError> {
        storage::nid(&env)
    }

    fn get_protocol_fee(env: Env) -> Result<u128, ContractError> {
        let protocol_fee = storage::protocol_fee(&env);

        Ok(protocol_fee)
    }

    fn get_fee_handler(env: Env) -> Result<Address, ContractError> {
        storage::get_fee_handler(&env)
    }

    fn get_deposit_id(env: Env) -> Result<u128, ContractError> {
        storage::deposit_id(&env)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        helpers::ensure_upgrade_authority(&env)?;
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        let current_version = storage::get_contract_version(&env);
        storage::set_contract_version(&env, current_version + 1);

        Ok(())
    }

    fn version(env: Env) -> u32 {
        storage::get_contract_version(&env)
    }
}
