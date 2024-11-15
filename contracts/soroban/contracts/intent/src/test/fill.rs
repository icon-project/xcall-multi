use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, IntoVal, String,
};

extern crate std;
use super::setup::TestContext;
use crate::{
    contract::IntentClient,
    event::{Message, OrderFilled},
    storage,
    types::{MessageType, OrderFill, OrderMessage},
};

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_fill_for_already_finished_order() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.dst_nid.clone());

    ctx.env.as_contract(&ctx.contract, || {
        let order_bytes = order.encode(&ctx.env);
        let order_hash = ctx.env.crypto().keccak256(&order_bytes);
        storage::store_finished_order(&ctx.env, &order_hash);
    });

    client.fill(&order, &ctx.admin, &ctx.admin.to_string());
}

#[test]
fn test_fill_order() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.dst_nid.clone());
    let dst_address = Address::from_string(&order.dst_address());
    let solver_address = Address::generate(&ctx.env);
    ctx.mint_native_token(&ctx.solver, 1000);

    let order_filled_event = OrderFilled {
        id: order.id(),
        srcNID: order.src_nid(),
    };

    let fill = OrderFill::new(
        order.id(),
        order.encode(&ctx.env),
        solver_address.to_string(),
    );
    let msg = OrderMessage::new(MessageType::FILL, fill.encode(&ctx.env));
    let message_event = Message {
        targetNetwork: order.src_nid(),
        sn: 1,
        msg: msg.encode(&ctx.env),
    };

    let protocol_fee = client.get_protocol_fee();
    let fee = (order.to_amount() * protocol_fee) / 10_000;
    let to_amount = order.to_amount() - fee;

    client.fill(&order, &ctx.solver, &solver_address.to_string());

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.solver.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    symbol_short!("fill"),
                    (
                        order.clone(),
                        ctx.solver.clone(),
                        solver_address.to_string()
                    )
                        .into_val(&ctx.env)
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            ctx.native_token.clone(),
                            symbol_short!("transfer"),
                            (ctx.solver.clone(), ctx.fee_handler.clone(), fee as i128)
                                .into_val(&ctx.env)
                        )),
                        sub_invocations: std::vec![]
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            ctx.native_token.clone(),
                            symbol_short!("transfer"),
                            (ctx.solver.clone(), dst_address.clone(), to_amount as i128)
                                .into_val(&ctx.env)
                        )),
                        sub_invocations: std::vec![]
                    }
                ]
            }
        ),]
    );

    let events = ctx.env.events().all();
    assert_eq!(
        vec![
            &ctx.env,
            events.get_unchecked(events.len() - 2),
            events.last_unchecked()
        ],
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("Message",).into_val(&ctx.env),
                message_event.into_val(&ctx.env)
            ),
            (
                client.address.clone(),
                ("OrderFilled",).into_val(&ctx.env),
                order_filled_event.into_val(&ctx.env)
            )
        ]
    );

    let order_hash = ctx.env.crypto().keccak256(&order.encode(&ctx.env));
    let finished_order = client.get_finished_order(&order_hash);
    assert_eq!(finished_order, true);

    let fee_handler_balance = ctx.get_native_token_balance(&ctx.fee_handler);
    assert_eq!(fee_handler_balance, fee);

    let dst_address_balance = ctx.get_native_token_balance(&dst_address);
    assert_eq!(dst_address_balance, to_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_resolve_fill_with_mismatched_order() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.mint_native_token(&ctx.solver, 1000);

    let solver_address = Address::generate(&ctx.env);
    let mut order = ctx.get_dummy_swap(ctx.nid.clone());

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_order(&ctx.env, order.id(), &order);
    });

    order.set_data(bytes!(&ctx.env, 0x11));

    client.fill(&order, &ctx.solver, &solver_address.to_string());
}

#[test]
fn test_resolve_fill() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let solver_address = Address::generate(&ctx.env);
    ctx.mint_native_token(&ctx.solver, 1000);
    ctx.mint_native_token(&ctx.contract, 100);

    let order = ctx.get_dummy_swap(ctx.nid.clone());

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_order(&ctx.env, order.id(), &order);
    });

    client.fill(&order, &ctx.solver, &solver_address.to_string());

    let contract_balance = ctx.get_native_token_balance(&ctx.contract);
    assert_eq!(contract_balance, 0);

    let solver_address_balance = ctx.get_native_token_balance(&solver_address);
    assert_eq!(solver_address_balance, order.amount());

    ctx.env.as_contract(&ctx.contract, || {
        let res = storage::get_order(&ctx.env, order.id());
        assert_eq!(res.is_err(), true)
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_resolve_fill_with_invalid_network_id() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let solver_address = Address::generate(&ctx.env);
    let order = ctx.get_dummy_swap(ctx.nid.clone());
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_order(&ctx.env, order.id(), &order);
    });

    let fill = OrderFill::new(
        order.id(),
        order.encode(&ctx.env),
        solver_address.to_string(),
    );
    let msg = OrderMessage::new(MessageType::FILL, fill.encode(&ctx.env));

    let conn_sn = 1;
    let src_network = String::from_str(&ctx.env, "solana");
    client.recv_message(&src_network, &conn_sn, &msg.encode(&ctx.env));

    let receipt = client.get_receipt(&src_network, &conn_sn);
    assert_eq!(receipt, true)
}
