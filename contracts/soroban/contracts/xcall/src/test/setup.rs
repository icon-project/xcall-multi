#![cfg(test)]

use soroban_sdk::{bytes, testutils::Address as _, token, vec, Address, Bytes, Env, String, Vec};
use soroban_xcall_lib::{
    messages::{
        call_message::CallMessage, call_message_persisted::CallMessagePersisted,
        call_message_rollback::CallMessageWithRollback, envelope::Envelope, msg_type::MessageType,
        AnyMessage,
    },
    network_address::NetworkAddress,
};

mod connection {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/centralized_connection.wasm"
    );
}

use crate::{
    contract::{Xcall, XcallClient},
    types::{message::InitializeMsg, request::CSMessageRequest, rollback::Rollback},
};

pub fn get_dummy_rollback(env: &Env) -> Rollback {
    let rollback = Rollback::new(
        Address::generate(&env),
        get_dummy_network_address(&env),
        Vec::new(&env),
        bytes!(&env, 0xabc),
        false,
    );
    rollback
}

pub fn get_dummy_message_request(env: &Env) -> CSMessageRequest {
    let from = get_dummy_network_address(&env);
    let to = String::from_str(&env, "hx9b79391cefc9a64dfda6446312ebb7717230df5b");
    let protocols = get_dummy_sources(&env);
    let msg_type = MessageType::CallMessage;
    let data = Bytes::new(&env);
    CSMessageRequest::new(from, to, 1, protocols, msg_type, data)
}

pub fn get_dummy_network_address(env: &Env) -> NetworkAddress {
    let network_id = String::from_str(&env, "stellar");
    let account = String::from_str(
        &env,
        "GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU",
    );

    NetworkAddress::new(&env, network_id, account)
}

pub fn get_dummy_call_msg(env: &Env) -> CallMessage {
    CallMessage {
        data: bytes!(&env, 0xabc),
    }
}

pub fn get_dummy_call_persisted_msg(env: &Env) -> CallMessagePersisted {
    CallMessagePersisted {
        data: bytes!(&env, 0xabc),
    }
}

pub fn get_dummy_call_rollback_msg(env: &Env) -> CallMessageWithRollback {
    CallMessageWithRollback {
        data: bytes!(&env, 0xabc),
        rollback: bytes!(&env, 0xabc),
    }
}

pub fn get_dummy_sources(env: &Env) -> Vec<String> {
    let sources = vec![
        &env,
        String::from_str(&env, "centralized_connection"),
        String::from_str(&env, "layerzero"),
    ];
    sources
}

pub fn get_dummy_destinations(env: &Env) -> Vec<String> {
    get_dummy_sources(&env)
}

pub fn get_dummy_protocols(env: &Env) -> Vec<String> {
    vec![
        &env,
        Address::generate(&env).to_string(),
        Address::generate(&env).to_string(),
        Address::generate(&env).to_string(),
    ]
}

pub fn get_dummy_envelope_msg(env: &Env, message: AnyMessage) -> Envelope {
    let sources = get_dummy_sources(&env);
    let destinations = get_dummy_destinations(&env);

    let envelope = Envelope {
        message,
        sources,
        destinations,
    };

    envelope
}

pub struct TestContext {
    pub contract: Address,
    pub admin: Address,
    pub fee_handler: Address,
    pub nid: String,
    pub env: Env,
    pub native_token: Address,
    pub token_admin: Address,
    pub network_address: NetworkAddress,
    pub upgrade_authority: Address,
    pub centralized_connection: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        let centralized_connection = env.register_contract_wasm(None, connection::WASM);

        Self {
            contract: env.register_contract(None, Xcall),
            admin: Address::generate(&env),
            fee_handler: Address::generate(&env),
            native_token: env.register_stellar_asset_contract(token_admin.clone()),
            nid: String::from_str(&env, "stellar"),
            network_address: get_dummy_network_address(&env),
            upgrade_authority: Address::generate(&env),
            env,
            token_admin,
            centralized_connection,
        }
    }

    pub fn init_context(&self, client: &XcallClient<'_>) {
        self.env.mock_all_auths();

        client.initialize(&InitializeMsg {
            sender: self.admin.clone(),
            network_id: String::from_str(&self.env, "icon"),
            native_token: self.native_token.clone(),
            upgrade_authority: self.upgrade_authority.clone(),
        });

        self.init_connection_state();
        client.set_protocol_fee(&100);
        client.set_default_connection(&self.nid, &self.centralized_connection);
    }

    pub fn init_connection_state(&self) {
        let connection_client = connection::Client::new(&self.env, &self.centralized_connection);

        let initialize_msg = connection::InitializeMsg {
            native_token: self.native_token.clone(),
            relayer: self.admin.clone(),
            xcall_address: self.contract.clone(),
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

    pub fn get_centralized_connection_fee(&self, need_response: bool) -> u128 {
        let connection_client = connection::Client::new(&self.env, &self.centralized_connection);
        let fee = connection_client.get_fee(&self.nid, &need_response);

        fee
    }
}
