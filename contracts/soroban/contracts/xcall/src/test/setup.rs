#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

extern crate std;

use crate::{
    contract::{Xcall, XcallClient},
    types::{message_types::InitializeMsg, network_address::NetId},
};

pub struct TestContext {
    pub contract: Address,
    pub admin: Address,
    pub fee_handler: Address,
    pub nid: NetId,
    pub env: Env,
    pub native_token: Address,
    pub token_admin: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);

        Self {
            contract: env.register_contract(None, Xcall),
            admin: Address::generate(&env),
            fee_handler: Address::generate(&env),
            native_token: env.register_stellar_asset_contract(token_admin.clone()),
            nid: NetId(String::from_str(&env, "icon")),
            env,
            token_admin,
        }
    }

    pub fn init_context(&self, client: &XcallClient<'_>) {
        self.env.mock_all_auths();

        client.initialize(&InitializeMsg {
            sender: self.admin.clone(),
            network_id: String::from_str(&self.env, "icon"),
            native_token: self.native_token.clone(),
        });
    }
}
