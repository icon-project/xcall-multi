use soroban_sdk::{
    bytes, symbol_short,
    testutils::{AuthorizedFunction, AuthorizedInvocation, Events},
    vec, IntoVal, String,
};

extern crate std;
use super::setup::TestContext;
use crate::{contract::IntentClient, event::SwapIntent, types::SwapOrder};

#[test]
#[should_panic(expected = "HostError: Error(Contract, #12)")]
fn test_swap_with_misconfigured_network_id() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let src_nid = String::from_str(&ctx.env, "icon");
    let order = SwapOrder::new(
        1,
        ctx.contract.to_string(),
        src_nid,
        String::from_str(&ctx.env, "solana"),
        ctx.admin.to_string(),
        ctx.admin.to_string(),
        ctx.native_token.to_string(),
        100,
        ctx.native_token.to_string(),
        100,
        bytes!(&ctx.env, 0x00),
    );

    client.swap(&order);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_swap_with_invalid_emitter_address() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = SwapOrder::new(
        1,
        ctx.admin.to_string(),
        ctx.nid,
        String::from_str(&ctx.env, "solana"),
        ctx.admin.to_string(),
        ctx.admin.to_string(),
        ctx.native_token.to_string(),
        100,
        ctx.native_token.to_string(),
        100,
        bytes!(&ctx.env, 0x00),
    );

    client.swap(&order);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_swap_with_insufficient_balance() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.dst_nid.clone());

    client.swap(&order);
}

#[test]
fn test_swap() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.mint_native_token(&ctx.admin, 100);

    let order = SwapOrder::new(
        1,
        ctx.contract.to_string(),
        ctx.nid.clone(),
        String::from_str(&ctx.env, "solana"),
        ctx.admin.to_string(),
        ctx.admin.to_string(),
        ctx.native_token.to_string(),
        100,
        ctx.native_token.to_string(),
        100,
        bytes!(&ctx.env, 0x00),
    );

    client.swap(&order);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    symbol_short!("swap"),
                    (order.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.native_token.clone(),
                        symbol_short!("transfer"),
                        (ctx.admin.clone(), ctx.contract.clone(), 100 as i128).into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                },]
            }
        ),]
    );

    let event_msg = SwapIntent {
        id: order.id(),
        emitter: order.emitter(),
        srcNID: order.src_nid(),
        dstNID: order.dst_nid(),
        creator: order.creator(),
        destinationAddress: order.dst_address(),
        token: order.token(),
        amount: order.amount(),
        toToken: order.to_token(),
        toAmount: order.to_amount(),
        data: order.data(),
    };
    let event = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        event,
        vec![
            &ctx.env,
            (
                ctx.contract.clone(),
                ("SwapIntent",).into_val(&ctx.env),
                event_msg.into_val(&ctx.env)
            )
        ]
    );

    // check if order is stored in the storage
    let current_deposit_id = client.get_deposit_id();
    let stored_order = client.get_order(&current_deposit_id);
    assert_eq!(stored_order.data(), order.data());

    // check contract and user balance
    let contract_balance = ctx.get_native_token_balance(&ctx.contract);
    let creator_balance = ctx.get_native_token_balance(&ctx.admin);
    assert_eq!(contract_balance, 100);
    assert_eq!(creator_balance, 0);
}
