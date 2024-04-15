#![cfg(test)]

extern crate std;

use crate::{
    contract::{CentralizedConnection, CentralizedConnectionClient},
    helpers::xcall,
    types::InitializeMsg,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    token, vec, Address, Bytes, Env, IntoVal, String, Symbol,
};

pub struct TestContext {
    env: Env,
    xcall: Address,
    contract: Address,
    relayer: Address,
    native_token: Address,
    token_admin: Address,
    nid: String,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        Self {
            xcall: env.register_contract_wasm(None, xcall::WASM),
            contract: env.register_contract(None, CentralizedConnection),
            relayer: Address::generate(&env),
            native_token: env.register_stellar_asset_contract(token_admin.clone()),
            nid: String::from_str(&env, "icon"),
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
        });
    }

    pub fn init_send_message(&self, client: &CentralizedConnectionClient<'static>) {
        self.init_context(&client);

        client.set_fee(&self.nid, &100, &100);
    }
}

fn get_dummy_initialize_msg(env: &Env) -> InitializeMsg {
    InitializeMsg {
        relayer: Address::generate(&env),
        native_token: env.register_stellar_asset_contract(Address::generate(&env)),
        xcall_address: Address::generate(&env),
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

    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);
    asset_client.mint(&ctx.xcall, &1000);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.send_message(&200, &ctx.nid, &1, &msg);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.xcall.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "send_message"),
                    (200_u128, ctx.nid.clone(), 1_i64, msg.clone()).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.native_token.clone(),
                        Symbol::new(&ctx.env, "transfer"),
                        (ctx.xcall.clone(), client.address.clone(), 200_i128).into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )]
    );

    let event = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        event,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("CentralizedConnection", "Message", ctx.nid, 1_u128).into_val(&ctx.env),
                msg.into_val(&ctx.env)
            )
        ]
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_send_message_fail_for_insufficient_fee() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_send_message(&client);

    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);
    asset_client.mint(&ctx.xcall, &1000);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.send_message(&150, &ctx.nid, &1, &msg);
}

#[test]
fn test_recv_message() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.recv_message(&ctx.nid, &1, &msg);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_recv_message_fail_for_duplicate_receipt() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.recv_message(&ctx.nid, &1, &msg);
    client.recv_message(&ctx.nid, &1, &msg);
}

#[test]
fn test_revert_message() {
    let ctx = TestContext::default();
    let client = CentralizedConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);
    client.revert_message(&10);
}
