use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, IntoVal, Symbol,
};

extern crate std;
use super::setup::TestContext;
use crate::contract::IntentClient;

mod intent {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/intent.wasm");
}

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let nid = client.get_nid();
    let admin = client.get_admin();
    let protocol_fee = client.get_protocol_fee();
    let fee_handler = client.get_fee_handler();
    let upgrade_authority = client.get_upgrade_authority();

    assert_eq!(nid, ctx.nid);
    assert_eq!(admin, ctx.admin);
    assert_eq!(protocol_fee, 100);
    assert_eq!(fee_handler, ctx.fee_handler);
    assert_eq!(upgrade_authority, ctx.upgrade_authority)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_initialize_fail_on_double_initialize() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);
    ctx.init_context(&client);
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);

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
fn test_set_protocol_fee() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

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
fn test_set_protocol_fee_handler() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    client.set_fee_handler(&ctx.fee_handler);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "set_fee_handler"),
                    (ctx.fee_handler.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_fee_handler(), ctx.fee_handler)
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
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
fn test_get_receipt() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let conn_sn = 10;
    let res = client.get_receipt(&ctx.nid, &conn_sn);
    assert_eq!(res, false)
}

#[test]
fn test_upgrade() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let wasm_hash = ctx.env.deployer().upload_contract_wasm(intent::WASM);
    assert_eq!(client.version(), 1);

    client.upgrade(&wasm_hash);
    assert_eq!(client.version(), 2);
}
