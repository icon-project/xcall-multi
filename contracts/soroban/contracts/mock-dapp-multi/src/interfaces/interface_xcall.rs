use soroban_sdk::{contractclient, Address, Env};

use crate::errors::ContractError;
use xcall::{messages::envelope::Envelope, types::network_address::NetworkAddress};

#[contractclient(name = "XcallClient")]
pub trait IXcall {
    fn send_call(
        env: Env,
        envelope: Envelope,
        to: NetworkAddress,
        fee: u128,
        sender: Address,
    ) -> Result<u128, ContractError>;
}
