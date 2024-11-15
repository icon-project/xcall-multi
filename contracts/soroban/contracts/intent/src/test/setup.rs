#![cfg(test)]

use soroban_sdk::{bytes, testutils::Address as _, token, Address, Env, String};

use crate::{
    contract::{Intent, IntentClient},
    types::SwapOrder,
};

pub mod intent {}

pub struct TestContext {
    pub contract: Address,
    pub admin: Address,
    pub fee_handler: Address,
    pub solver: Address,
    pub nid: String,
    pub dst_nid: String,
    pub env: Env,
    pub native_token: Address,
    pub token_admin: Address,
    pub upgrade_authority: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);

        Self {
            contract: env.register_contract(None, Intent),
            admin: Address::generate(&env),
            fee_handler: Address::generate(&env),
            solver: Address::generate(&env),
            native_token: env.register_stellar_asset_contract(token_admin.clone()),
            nid: String::from_str(&env, "stellar"),
            dst_nid: String::from_str(&env, "solana"),
            upgrade_authority: Address::generate(&env),
            env,
            token_admin,
        }
    }

    pub fn init_context(&self, client: &IntentClient<'_>) {
        self.env.mock_all_auths();

        client.initialize(
            &self.nid.clone(),
            &self.admin.clone(),
            &self.fee_handler.clone(),
            &self.native_token.clone(),
            &self.upgrade_authority.clone(),
        );

        client.set_protocol_fee(&100);
    }

    pub fn get_dummy_swap(&self, dst_nid: String) -> SwapOrder {
        SwapOrder::new(
            1,
            self.contract.to_string(),
            self.nid.clone(),
            dst_nid,
            Address::generate(&self.env).to_string(),
            Address::generate(&self.env).to_string(),
            self.native_token.to_string(),
            100,
            self.native_token.to_string(),
            100,
            bytes!(&self.env, 0x00),
        )
    }

    pub fn mint_native_token(&self, address: &Address, amount: u128) {
        let native_token_client = token::StellarAssetClient::new(&self.env, &self.native_token);
        native_token_client.mint(&address, &(*&amount as i128));
    }

    pub fn get_native_token_balance(&self, address: &Address) -> u128 {
        let native_token_client = token::TokenClient::new(&self.env, &self.native_token);
        let balance = native_token_client.balance(address);

        *&balance as u128
    }
}
