use soroban_sdk::{testutils::Address as _, token, Address, Env, String};
use soroban_xcall_lib::network_address::NetworkAddress;

use crate::contract::{MockDapp, MockDappClient};

mod connection {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/centralized_connection.wasm"
    );
}

mod xcall_module {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/xcall.wasm");
}

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
    pub admin: Address,
    pub network_address: NetworkAddress,
    pub env: Env,
    pub native_token: Address,
    pub xcall: Address,
    pub upgrade_authority: Address,
    pub centralized_connection: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let address = Address::generate(&env);

        Self {
            contract: env.register_contract(None, MockDapp),
            nid: String::from_str(&env, "stellar"),
            admin: Address::generate(&env),
            native_token: env.register_stellar_asset_contract(address),
            network_address: get_dummy_network_address(&env),
            xcall: env.register_contract_wasm(None, xcall_module::WASM),
            upgrade_authority: Address::generate(&env),
            centralized_connection: env.register_contract_wasm(None, connection::WASM),
            env,
        }
    }

    pub fn init_context(&self, client: &MockDappClient<'_>) {
        self.env.mock_all_auths();

        client.init(&self.admin, &self.xcall, &self.native_token);
        client.add_connection(
            &self.centralized_connection.to_string(),
            &Address::generate(&self.env).to_string(),
            &self.nid,
        );

        self.init_xcall_state();
        self.init_connection_state();
    }

    pub fn init_xcall_state(&self) {
        let xcall_client = xcall_module::Client::new(&self.env, &self.xcall);

        let initialize_msg = xcall_module::InitializeMsg {
            native_token: self.native_token.clone(),
            network_id: self.nid.clone(),
            sender: Address::generate(&self.env),
            upgrade_authority: self.upgrade_authority.clone(),
        };
        xcall_client.initialize(&initialize_msg);

        xcall_client.set_protocol_fee(&100_u128);
        xcall_client.set_default_connection(&self.nid, &self.centralized_connection)
    }

    pub fn init_connection_state(&self) {
        let connection_client = connection::Client::new(&self.env, &self.centralized_connection);

        let initialize_msg = connection::InitializeMsg {
            native_token: self.native_token.clone(),
            relayer: Address::generate(&self.env),
            xcall_address: self.xcall.clone(),
            upgrade_authority: self.upgrade_authority.clone(),
        };
        connection_client.initialize(&initialize_msg);

        let message_fee = 100;
        let response_fee = 100;
        connection_client.set_fee(&self.nid, &message_fee, &response_fee);
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
