use soroban_sdk::{Address, Bytes, BytesN, Env, String};

use crate::{error::ContractError, types::swap_order::SwapOrder};
pub trait IIntent {
    fn initialize(
        env: Env,
        network_id: String,
        admin: Address,
        fee_handler: Address,
        native_token: Address,
        upgrade_authority: Address,
    ) -> Result<(), ContractError>;

    fn swap(env: Env, order: SwapOrder) -> Result<(), ContractError>;

    fn fill(
        env: Env,
        order: SwapOrder,
        sender: Address,
        solver_address: String,
    ) -> Result<(), ContractError>;

    fn cancel(env: Env, id: u128) -> Result<(), ContractError>;

    fn recv_message(
        env: Env,
        src_network: String,
        conn_sn: u128,
        msg: Bytes,
    ) -> Result<(), ContractError>;

    fn set_admin(env: Env, address: Address) -> Result<(), ContractError>;

    fn set_fee_handler(env: Env, address: Address) -> Result<(), ContractError>;

    fn set_protocol_fee(env: Env, fee: u128) -> Result<(), ContractError>;

    fn set_upgrade_authority(env: &Env, address: Address) -> Result<(), ContractError>;

    fn get_order(env: Env, id: u128) -> Result<SwapOrder, ContractError>;

    fn get_finished_order(env: Env, bytes: BytesN<32>) -> Result<bool, ContractError>;

    fn get_receipt(env: Env, network_id: String, conn_sn: u128) -> bool;

    fn get_admin(env: Env) -> Result<Address, ContractError>;

    fn get_upgrade_authority(env: Env) -> Result<Address, ContractError>;

    fn get_nid(env: Env) -> Result<String, ContractError>;

    fn get_protocol_fee(env: Env) -> Result<u128, ContractError>;

    fn get_fee_handler(env: Env) -> Result<Address, ContractError>;

    fn get_deposit_id(env: Env) -> Result<u128, ContractError>;

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError>;

    fn version(env: Env) -> u32;
}
