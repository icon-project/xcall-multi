use soroban_sdk::{contractclient, Address, Env, String, Vec};

use crate::errors::ContractError;
use xcall::{messages::envelope::Envelope, types::network_address::NetworkAddress};

#[contractclient(name = "XcallClient")]
pub trait IXcall {
    fn send_call(
        env: Env,
        tx_origin: Address,
        sender: Address,
        envelope: Envelope,
        to: NetworkAddress,
    ) -> Result<u128, ContractError>;

    fn get_fee(
        env: Env,
        nid: String,
        rollback: bool,
        sources: Option<Vec<String>>,
    ) -> Result<u128, ContractError>;
}
