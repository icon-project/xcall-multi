use soroban_sdk::{Address, Bytes, Env, String};

use crate::error::ContractError;

pub trait IGeneralizedConnection {
    fn set_admin(env: &Env, address: Address) -> Result<(), ContractError>;

    fn admin(env: &Env) -> Result<Address, ContractError>;

    fn get_receipt(env: &Env, src_network: String, conn_sn: u128) -> bool;

    fn send_message(env: &Env, to: String, msg: Bytes);

    fn recv_message(env: &Env, src_network: String, conn_sn: u128) -> Result<(), ContractError>;
}
