use soroban_sdk::{contractclient, Address, Env, String, Vec};
use soroban_xcall_lib::messages::envelope::Envelope;

use crate::errors::ContractError;

#[contractclient(name = "XcallClient")]
pub trait IXcall {
    fn send_call(
        env: Env,
        tx_origin: Address,
        sender: Address,
        envelope: Envelope,
        to: String,
    ) -> Result<u128, ContractError>;

    fn get_fee(
        env: Env,
        nid: String,
        rollback: bool,
        sources: Option<Vec<String>>,
    ) -> Result<u128, ContractError>;
}
