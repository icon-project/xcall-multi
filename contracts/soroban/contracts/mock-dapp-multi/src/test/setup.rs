use soroban_sdk::{Address, Env, String};
use xcall::types::network_address::NetworkAddress;

use crate::contract::MockDapp;

pub fn get_dummy_network_address(env: &Env) -> NetworkAddress {
    let network_id = String::from_str(&env, "stellar");
    let account = String::from_str(
        &env,
        "GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU",
    );

    NetworkAddress::new(&env, network_id, account)
}

pub struct TestContext {
    pub contract: Address,
    pub nid: String,
    pub network_address: NetworkAddress,
    pub env: Env,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();

        Self {
            contract: env.register_contract(None, MockDapp),
            nid: String::from_str(&env, "stellar"),
            network_address: get_dummy_network_address(&env),
            env,
        }
    }
}
