#![cfg(test)]

extern crate std;

use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Bytes, IntoVal, String,
};
use soroban_xcall_lib::messages::{
    call_message::CallMessage, call_message_rollback::CallMessageWithRollback, envelope::Envelope,
    AnyMessage,
};

use super::setup::*;
use crate::{
    contract::{Xcall, XcallClient},
    send_message, storage,
};

#[test]
fn test_send_call_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let need_response = false;
    let sender = Address::generate(&ctx.env);
    let tx_origin = Address::generate(&ctx.env);

    let mint_amount = 500;
    ctx.mint_native_token(&tx_origin, mint_amount);

    let sources = vec![&ctx.env, ctx.centralized_connection.to_string()];
    let destinations = vec![&ctx.env];
    let envelope = Envelope {
        sources: sources.clone(),
        destinations: destinations.clone(),
        message: AnyMessage::CallMessage(CallMessage {
            data: bytes!(&ctx.env, 0xabc),
        }),
    };

    let protocol_fee = client.get_protocol_fee();
    let fee = client.get_fee(&ctx.nid, &need_response, &Some(sources.clone()));
    let connection_fee = fee - protocol_fee;

    let res = client.send_call(
        &tx_origin,
        &sender,
        &envelope,
        &ctx.network_address.to_string(),
    );
    assert_eq!(res, 1);
    assert_eq!(
        ctx.env.auths(),
        std::vec![
            (
                sender.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.contract.clone(),
                        symbol_short!("send_call"),
                        (
                            &tx_origin,
                            &sender,
                            Envelope {
                                sources: sources.clone(),
                                destinations,
                                message: AnyMessage::CallMessage(CallMessage {
                                    data: bytes!(&ctx.env, 0xabc),
                                }),
                            },
                            ctx.network_address.to_string().clone()
                        )
                            .into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                tx_origin.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.contract.clone(),
                        symbol_short!("send_call"),
                        (
                            &tx_origin,
                            &sender,
                            envelope,
                            ctx.network_address.to_string().clone()
                        )
                            .into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![
                        AuthorizedInvocation {
                            function: AuthorizedFunction::Contract((
                                ctx.native_token.clone(),
                                symbol_short!("transfer"),
                                (
                                    tx_origin.clone(),
                                    ctx.centralized_connection.clone(),
                                    connection_fee as i128
                                )
                                    .into_val(&ctx.env)
                            )),
                            sub_invocations: std::vec![]
                        },
                        AuthorizedInvocation {
                            function: AuthorizedFunction::Contract((
                                ctx.native_token.clone(),
                                symbol_short!("transfer"),
                                (tx_origin.clone(), ctx.admin.clone(), protocol_fee as i128)
                                    .into_val(&ctx.env)
                            )),
                            sub_invocations: std::vec![]
                        },
                    ]
                }
            )
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        let sn = storage::get_next_sn(&ctx.env);
        assert_eq!(sn, res + 1)
    });

    let fee_handler_balance = ctx.get_native_token_balance(&ctx.admin);
    assert_eq!(fee_handler_balance, protocol_fee);

    let tx_origin_balance = ctx.get_native_token_balance(&tx_origin);
    assert_eq!(tx_origin_balance, mint_amount - fee);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_message_with_greater_than_max_data_size() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let mut bytes = Bytes::new(&ctx.env);
    bytes.copy_from_slice(2050, &[1; 2050]);

    let tx_origin = Address::generate(&ctx.env);
    let msg = AnyMessage::CallMessage(CallMessage { data: bytes });
    let envelope = get_dummy_envelope_msg(&ctx.env, msg);

    client.send_call(
        &tx_origin,
        &ctx.admin,
        &envelope,
        &ctx.network_address.to_string(),
    );
}

#[test]
#[should_panic(expected = "RollbackNotPossible")]
fn test_process_rollback_message_with_invalid_contract_address() {
    let ctx = TestContext::default();

    let sender = Address::from_string(&String::from_str(
        &ctx.env,
        "GCX7EUFDXJUZEWHT5UGH2ZISTKXSUQSHFKHJMNWCK6JIQ2PX5BPJHOLU",
    ));

    let rollback_msg = get_dummy_call_rollback_msg(&ctx.env);
    let message = AnyMessage::CallMessageWithRollback(rollback_msg);
    let envelope = &get_dummy_envelope_msg(&ctx.env, message);

    send_message::process_message(&ctx.env, &ctx.network_address, 1, &sender, envelope).unwrap();
}

#[test]
#[should_panic(expected = "MaxRollbackSizeExceeded")]
fn test_process_rollback_message_with_greater_than_max_rollback_size() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let mut bytes = Bytes::new(&ctx.env);
    bytes.copy_from_slice(1025, &[0; 1025]);

    let rollback_msg = CallMessageWithRollback {
        data: bytes!(&ctx.env, 0xab),
        rollback: bytes,
    };
    let message = AnyMessage::CallMessageWithRollback(rollback_msg);
    let envelope = &get_dummy_envelope_msg(&ctx.env, message);

    send_message::process_message(&ctx.env, &ctx.network_address, 1, &ctx.contract, envelope)
        .unwrap();
}

#[test]
#[should_panic(expected = "NoRollbackData")]
fn test_process_rollback_message_with_empty_rollback_data() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let rollback_msg = CallMessageWithRollback {
        data: bytes!(&ctx.env, 0xab),
        rollback: Bytes::new(&ctx.env),
    };
    let message = AnyMessage::CallMessageWithRollback(rollback_msg);
    let envelope = &get_dummy_envelope_msg(&ctx.env, message);

    send_message::process_message(&ctx.env, &ctx.network_address, 1, &ctx.contract, envelope)
        .unwrap();
}

#[test]
fn test_process_rollback_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let rollback_msg = CallMessageWithRollback {
        data: bytes!(&ctx.env, 0xab),
        rollback: bytes!(&ctx.env, 0xab),
    };
    let message = AnyMessage::CallMessageWithRollback(rollback_msg);
    let envelope = &get_dummy_envelope_msg(&ctx.env, message);

    ctx.env.as_contract(&client.address, || {
        let res = send_message::process_message(
            &ctx.env,
            &ctx.network_address,
            1,
            &ctx.contract,
            envelope,
        );
        let rollback = storage::get_rollback(&ctx.env, 1);
        assert!(rollback.is_ok());
        assert!(res.is_ok())
    });
}

#[test]
fn test_call_connection_for_rollback_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths_allowing_non_root_auth();

    let sender = Address::generate(&ctx.env);
    let msg = Bytes::new(&ctx.env);
    let sources = vec![&ctx.env, ctx.centralized_connection.to_string()];

    let need_response = true;
    let fee = ctx.get_centralized_connection_fee(need_response);
    ctx.mint_native_token(&sender, fee);

    send_message::call_connection(
        &ctx.env,
        &sender,
        &ctx.nid,
        1,
        sources.clone(),
        need_response,
        msg.clone(),
    )
    .unwrap();

    let sender_balance = ctx.get_native_token_balance(&sender);
    let connection_balance = ctx.get_native_token_balance(&ctx.centralized_connection);

    assert_eq!(sender_balance, 0);
    assert_eq!(connection_balance, fee);
}

#[test]
fn test_call_connection_for_call_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths_allowing_non_root_auth();

    let sender = Address::generate(&ctx.env);
    let msg = Bytes::new(&ctx.env);
    let sources = vec![&ctx.env, ctx.centralized_connection.to_string()];

    let need_response = false;
    let fee = ctx.get_centralized_connection_fee(need_response);
    ctx.mint_native_token(&sender, fee);

    send_message::call_connection(
        &ctx.env,
        &sender,
        &ctx.nid,
        1,
        sources.clone(),
        need_response,
        msg.clone(),
    )
    .unwrap();

    let sender_balance = ctx.get_native_token_balance(&ctx.contract);
    let connection_balance = ctx.get_native_token_balance(&ctx.centralized_connection);

    assert_eq!(sender_balance, 0);
    assert_eq!(connection_balance, fee);
}

#[test]
fn test_calim_protocol_fee() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths_allowing_non_root_auth();

    let protocol_fee = 100;
    let sender = Address::generate(&ctx.env);

    ctx.mint_native_token(&sender, protocol_fee);

    ctx.env.as_contract(&ctx.contract, || {
        send_message::claim_protocol_fee(&ctx.env, &sender).unwrap();

        let fee_handler_balance = ctx.get_native_token_balance(&ctx.admin);
        let sender_balance = ctx.get_native_token_balance(&sender);

        assert_eq!(fee_handler_balance, protocol_fee);
        assert_eq!(sender_balance, 0);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_claim_protocol_fail_for_insufficient_amount_sent() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths_allowing_non_root_auth();

    let sender = Address::generate(&ctx.env);
    ctx.mint_native_token(&sender, 100);

    ctx.env.as_contract(&ctx.contract, || {
        Xcall::set_protocol_fee(&ctx.env, 150).unwrap();
        send_message::claim_protocol_fee(&ctx.env, &sender).unwrap();
    });
}
