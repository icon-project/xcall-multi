#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, IntoVal, String, Symbol, Vec,
};

extern crate std;

use super::setup::*;
use crate::contract::XcallClient;

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    assert_eq!(client.get_admin(), ctx.admin.clone());
    assert_eq!(client.get_protocol_fee_handler(), ctx.admin.clone())
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_initialize_fail_on_double_initialize() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);
    ctx.init_context(&client);
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let new_admin = Address::generate(&ctx.env);
    client.set_admin(&new_admin);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "set_admin"),
                    (new_admin.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_admin(), new_admin)
}

#[test]
fn test_protocol_fee() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    client.set_protocol_fee_handler(&ctx.fee_handler);
    client.set_protocol_fee(&100);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "set_protocol_fee"),
                    (100_u128,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_protocol_fee(), 100);
}

#[test]
fn test_protocol_fee_handler() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    client.set_protocol_fee_handler(&ctx.fee_handler);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "set_protocol_fee_handler"),
                    (ctx.fee_handler.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_protocol_fee_handler(), ctx.fee_handler)
}

#[test]
fn test_verify_success_response() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    assert_eq!(client.verify_success(&1), false)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_get_default_connection_fail() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    client.get_default_connection(&String::from_str(&ctx.env, "cosmos"));
}

#[test]
fn test_set_default_connection() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let default_connection = Address::generate(&ctx.env);
    client.set_default_connection(&ctx.nid, &default_connection);
    assert_eq!(client.get_default_connection(&ctx.nid), default_connection)
}

#[test]
fn test_get_fee() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let need_response = true;
    let sources: Vec<String> = vec![&ctx.env];

    let protocol_fee = client.get_protocol_fee();
    let centralized_conn_fee = ctx.get_centralized_connection_fee(need_response);
    let fee = client.get_fee(&ctx.nid, &need_response, &Some(sources));
    assert_eq!(fee, protocol_fee + centralized_conn_fee)
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
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
