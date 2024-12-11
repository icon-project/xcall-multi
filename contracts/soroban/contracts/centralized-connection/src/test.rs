#![cfg(test)]

extern crate std;

mod xcall_module {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/xcall.wasm");
}
mod connection {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/centralized_connection.wasm"
    );
}

use crate::{
    contract::{CentralizedConnection, CentralizedConnectionClient},
    event::SendMsgEvent,
    storage,
    types::InitializeMsg,
};
use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    token, vec, Address, Bytes, Env, IntoVal, String, Symbol, Vec,
};
use soroban_xcall_lib::{messages::msg_type::MessageType, network_address::NetworkAddress};
use xcall::{
    storage as xcall_storage,
    types::{message::CSMessage, request::CSMessageRequest, rollback::Rollback},
};

pub struct TestContext {
    env: Env,
    xcall: Address,
    contract: Address,
    relayer: Address,
    native_token: Address,
    token_admin: Address,
    nid: String,
    upgrade_authority: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        let native_token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        Self {
            xcall: env.register_contract_wasm(None, xcall_module::WASM),
            contract: env.register_contract(None, CentralizedConnection),
            relayer: Address::generate(&env),
            native_token: native_token_contract.address(),
            nid: String::from_str(&env, "icon"),
            upgrade_authority: Address::generate(&env),
            env,
            token_admin,
        }
    }

    pub fn init_context(&self, client: &CentralizedConnectionClient<'static>) {
        self.env.mock_all_auths();

        client.initialize(&InitializeMsg {
            relayer: self.relayer.clone(),
            native_token: self.native_token.clone(),
            xcall_address: self.xcall.clone(),
            upgrade_authority: self.upgrade_authority.clone(),
        });

        self.init_xcall_state();
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
        xcall_client.set_default_connection(&self.nid, &self.contract)
    }

    pub fn init_send_message(&self, client: &CentralizedConnectionClient<'static>) {
        self.init_context(&client);
        self.env.mock_all_auths_allowing_non_root_auth();

        client.set_fee(&self.nid, &100, &100);
    }
}

fn get_dummy_initialize_msg(env: &Env) -> InitializeMsg {
    let native_token_contract = env.register_stellar_asset_contract_v2(Address::generate(&env));

    InitializeMsg {
        relayer: Address::generate(&env),
        native_token: native_token_contract.address(),
        xcall_address: Address::generate(&env),
        upgrade_authority: Address::generate(&env),
    }
}

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let admin = client.get_admin();
    assert_eq!(admin, ctx.relayer)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_initialize_fail_on_double_initialize() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    client.initialize(&get_dummy_initialize_msg(&ctx.env));
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let new_admin = Address::generate(&ctx.env);
    client.set_admin(&new_admin);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.relayer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("set_admin"),
                    (new_admin.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    )
}

#[test]
#[should_panic]
fn test_set_admin_fail() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let new_admin = Address::generate(&ctx.env);
    client.set_admin(&new_admin);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.xcall,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("set_admin"),
                    (new_admin.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    )
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let new_upgrade_authority = Address::generate(&ctx.env);
    client.set_upgrade_authority(&new_upgrade_authority);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.upgrade_authority.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    Symbol::new(&ctx.env, "set_upgrade_authority"),
                    (&new_upgrade_authority,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    let autorhity = client.get_upgrade_authority();
    assert_eq!(autorhity, new_upgrade_authority);
}

#[test]
fn test_set_fee() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let nid = String::from_str(&ctx.env, "icon");
    client.set_fee(&nid, &10, &10);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.relayer,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("set_fee"),
                    (nid.clone(), 10_u128, 10_u128).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_fee(&nid, &true), 20);
    assert_eq!(client.get_fee(&nid, &false), 10);
}

#[test]
fn test_claim_fees() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let token_client = token::Client::new(&ctx.env, &ctx.native_token);
    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);

    asset_client.mint(&ctx.contract, &1000);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.token_admin,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.native_token.clone(),
                    symbol_short!("mint"),
                    (&ctx.contract.clone(), 1000_i128,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token_client.balance(&ctx.contract), 1000);

    client.claim_fees();
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.relayer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "claim_fees"),
                    ().into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token_client.balance(&ctx.relayer), 1000);
    assert_eq!(token_client.balance(&ctx.contract), 0);
    assert_eq!(ctx.env.auths(), std::vec![]);
}

#[test]
fn test_send_message() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_send_message(&client);

    let tx_origin = Address::generate(&ctx.env);

    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);
    asset_client.mint(&tx_origin, &1000);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.send_message(&tx_origin, &ctx.nid, &1, &msg);

    assert_eq!(
        ctx.env.auths(),
        std::vec![
            (
                ctx.xcall.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        client.address.clone(),
                        Symbol::new(&ctx.env, "send_message"),
                        (tx_origin.clone(), ctx.nid.clone(), 1_i64, msg.clone()).into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                tx_origin.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.native_token.clone(),
                        Symbol::new(&ctx.env, "transfer"),
                        (tx_origin.clone(), ctx.contract.clone(), 200_i128).into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            )
        ]
    );

    let emit_msg = SendMsgEvent {
        targetNetwork: ctx.nid.clone(),
        connSn: 1_u128,
        msg: msg.clone(),
    };
    let event = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        event,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("Message",).into_val(&ctx.env),
                emit_msg.into_val(&ctx.env)
            )
        ]
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_send_message_fail_for_insufficient_fee() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_send_message(&client);

    let sender = Address::generate(&ctx.env);

    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);
    asset_client.mint(&sender, &100);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.send_message(&sender, &ctx.nid, &1, &msg);
}

#[test]
fn test_get_receipt_returns_false() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    let sequence_no = 1;
    let receipt = client.get_receipt(&ctx.nid, &sequence_no);
    assert_eq!(receipt, false);

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_receipt(&ctx.env, ctx.nid.clone(), sequence_no);
    });

    let receipt = client.get_receipt(&ctx.nid, &sequence_no);
    assert_eq!(receipt, true)
}

#[test]
fn test_recv_message() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let protocols: Vec<String> = vec![&ctx.env, ctx.contract.to_string()];
    let from = NetworkAddress::new(
        &ctx.env,
        String::from_str(&ctx.env, "0x2.icon"),
        ctx.xcall.to_string(),
    );
    let request = CSMessageRequest::new(
        from,
        Address::generate(&ctx.env).to_string(),
        1,
        protocols,
        MessageType::CallMessagePersisted,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_request(&ctx.env, &request);
    let encoded = cs_message.encode(&ctx.env);

    let conn_sn = 1;
    let from_nid = String::from_str(&ctx.env, "0x2.icon");
    client.recv_message(&from_nid, &conn_sn, &encoded);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_recv_message_duplicate_connection_sequence() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let protocols: Vec<String> = vec![&ctx.env, ctx.contract.to_string()];
    let from = NetworkAddress::new(
        &ctx.env,
        String::from_str(&ctx.env, "0x2.icon"),
        ctx.xcall.to_string(),
    );
    let request = CSMessageRequest::new(
        from,
        Address::generate(&ctx.env).to_string(),
        1,
        protocols,
        MessageType::CallMessagePersisted,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_request(&ctx.env, &request);
    let encoded = cs_message.encode(&ctx.env);

    let conn_sn = 1;
    let from_nid = String::from_str(&ctx.env, "0x2.icon");
    client.recv_message(&from_nid, &conn_sn, &encoded);

    client.recv_message(&from_nid, &conn_sn, &encoded);
}

#[test]
pub fn test_revert_message() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sequence_no = 1;
    let protocols: Vec<String> = vec![&ctx.env, ctx.contract.to_string()];
    let to = NetworkAddress::new(
        &ctx.env,
        String::from_str(&ctx.env, "0x2.icon"),
        ctx.xcall.to_string(),
    );
    let rollback = Rollback::new(
        Address::generate(&ctx.env),
        to,
        protocols.clone(),
        bytes!(&ctx.env, 0xabc),
        false,
    );
    ctx.env.as_contract(&ctx.xcall, || {
        xcall_storage::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    client.revert_message(&sequence_no);

    ctx.env.as_contract(&ctx.xcall, || {
        // rollback should be enabled
        let rollback = xcall_storage::get_rollback(&ctx.env, sequence_no).unwrap();
        assert_eq!(rollback.enabled, true);
    });
}

#[test]
fn test_upgrade() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let wasm_hash = ctx.env.deployer().upload_contract_wasm(connection::WASM);
    assert_eq!(client.version(), 1);

    client.upgrade(&wasm_hash);
    assert_eq!(client.version(), 2);
}
