use soroban_sdk::{contractclient, Bytes, Env, String};

use crate::errors::ContractError;

#[contractclient(name = "CentralizedConnectionClient")]
pub trait ICentralizedConnection {
    fn send_message(
        env: Env,
        amount: u128,
        to: String,
        sn: i64,
        msg: Bytes,
    ) -> Result<(), ContractError>;

    fn get_fee(env: Env, network_id: String, response: bool) -> u128;
}
