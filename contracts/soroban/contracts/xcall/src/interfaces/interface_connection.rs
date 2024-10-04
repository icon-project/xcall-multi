use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::errors::ContractError;

#[contractclient(name = "ConnectionClient")]
pub trait IConnection {
    fn send_message(
        env: Env,
        tx_origin: Address,
        to: String,
        sn: i64,
        msg: Bytes,
    ) -> Result<(), ContractError>;

    fn get_fee(env: Env, network_id: String, response: bool) -> Result<u128, ContractError>;
}
