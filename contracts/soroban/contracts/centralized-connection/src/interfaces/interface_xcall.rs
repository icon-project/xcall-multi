use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::errors::ContractError;

#[contractclient(name = "XcallClient")]
pub trait IXcall {
    fn handle_message(
        env: Env,
        sender: Address,
        from_nid: String,
        msg: Bytes,
    ) -> Result<(), ContractError>;

    fn handle_error(env: Env, sender: Address, sequence_no: u128) -> Result<(), ContractError>;
}
