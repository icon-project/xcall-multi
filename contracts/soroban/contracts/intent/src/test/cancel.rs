use soroban_sdk::{
    symbol_short,
    testutils::{AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, IntoVal, String,
};

extern crate std;
use super::setup::TestContext;
use crate::{
    contract::IntentClient,
    event::{Message, OrderCancelled},
    storage,
    types::{Cancel, MessageType, OrderFill, OrderMessage},
};

#[test]
fn test_cancel_order() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.dst_nid.clone());
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_order(&ctx.env, order.id(), &order);
    });

    client.cancel(&order.id());

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            Address::from_string(&order.creator()),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    symbol_short!("cancel"),
                    (order.id(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        ),]
    );

    let cancel = Cancel::new(order.encode(&ctx.env));
    let msg = OrderMessage::new(MessageType::CANCEL, cancel.encode(&ctx.env));
    let message_event = Message {
        targetNetwork: order.dst_nid(),
        sn: 1,
        msg: msg.encode(&ctx.env),
    };
    let events = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("Message",).into_val(&ctx.env),
                message_event.into_val(&ctx.env)
            ),
        ]
    );
}

#[test]
fn test_resolve_cancel_in_same_source_and_destination_chain() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.nid.clone());
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_order(&ctx.env, order.id(), &order);
    });

    client.cancel(&order.id());

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            Address::from_string(&order.creator()),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    symbol_short!("cancel"),
                    (order.id(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        ),]
    );

    let fill = OrderFill::new(order.id(), order.encode(&ctx.env), order.creator());
    let msg = OrderMessage::new(MessageType::FILL, fill.encode(&ctx.env));
    let message_event = Message {
        targetNetwork: order.src_nid(),
        sn: 1,
        msg: msg.encode(&ctx.env),
    };
    let order_cancel_event = OrderCancelled {
        id: order.id(),
        srcNID: order.src_nid(),
    };
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
                ("OrderCancelled",).into_val(&ctx.env),
                order_cancel_event.into_val(&ctx.env)
            )
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        let order_bytes = order.encode(&ctx.env);
        let order_hash = ctx.env.crypto().keccak256(&order_bytes);
        let filled_order = storage::order_finished(&ctx.env, &order_hash);
        assert_eq!(filled_order, true);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_resolve_cancel_with_invalid_network_id() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.dst_nid.clone());
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_order(&ctx.env, order.id(), &order);
    });

    let cancel = Cancel::new(order.encode(&ctx.env));
    let msg = OrderMessage::new(MessageType::CANCEL, cancel.encode(&ctx.env));

    let conn_sn = 1;
    let src_network = String::from_str(&ctx.env, "solana");
    client.recv_message(&src_network, &conn_sn, &msg.encode(&ctx.env));

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    symbol_short!("cancel"),
                    (order.id(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        ),]
    );

    let receipt = client.get_receipt(&src_network, &conn_sn);
    assert_eq!(receipt, true);

    let order_hash = ctx.env.crypto().keccak256(&order.encode(&ctx.env));
    ctx.env.as_contract(&ctx.contract, || {
        let filled_order = storage::order_finished(&ctx.env, &order_hash);
        assert_eq!(filled_order, true)
    });
}

#[test]
fn test_resolve_cancel_for_already_filled_order() {
    let ctx = TestContext::default();
    let client = IntentClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let order = ctx.get_dummy_swap(ctx.dst_nid.clone());
    let order_hash = ctx.env.crypto().keccak256(&order.encode(&ctx.env));
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_finished_order(&ctx.env, &order_hash);
    });

    let cancel = Cancel::new(order.encode(&ctx.env));
    let msg = OrderMessage::new(MessageType::CANCEL, cancel.encode(&ctx.env));

    let conn_sn = 1;
    let src_network = String::from_str(&ctx.env, "solana");
    let res = client.recv_message(&src_network, &conn_sn, &msg.encode(&ctx.env));
    assert_eq!(res, ());
}
