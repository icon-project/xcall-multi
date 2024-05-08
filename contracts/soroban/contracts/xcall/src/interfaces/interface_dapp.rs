use soroban_sdk::{contractclient, Address, Bytes, Env, String, Vec};

use crate::types::network_address::NetworkAddress;

#[contractclient(name = "DappClient")]
pub trait IDapp {
    fn handle_call_message(
        env: Env,
        sender: Address,
        from: NetworkAddress,
        data: Bytes,
        protocols: Option<Vec<String>>,
    );
}
