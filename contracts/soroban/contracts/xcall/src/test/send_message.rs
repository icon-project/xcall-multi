#![cfg(test)]

extern crate std;

use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Bytes, IntoVal, String, Vec,
};

use super::setup::*;
use crate::contract::{Xcall, XcallClient};
use crate::messages::envelope::Envelope;
use crate::messages::{
    call_message::CallMessage, call_message_rollback::CallMessageWithRollback, AnyMessage,
};

#[test]
fn test_send_call_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths_allowing_non_root_auth();

    let sent_fee = 300;
    let need_response = false;
    let sender = Address::generate(&ctx.env);

    let mint_amount = 500;
    ctx.mint_native_token(&sender, mint_amount);

    let sources = vec![&ctx.env, ctx.centralized_connection.to_string()];
    let destinations = vec![&ctx.env];
    let envelope = Envelope {
        sources: sources.clone(),
        destinations,
        message: AnyMessage::CallMessage(CallMessage {
            data: bytes!(&ctx.env, 0xabc),
        }),
    };

    let protocol_fee = client.get_protocol_fee();
    let fee = client.get_fee(&ctx.nid, &need_response, &Some(sources));
    let connection_fee = fee - protocol_fee;

    let res = client.send_call(&envelope, &ctx.network_address, &sent_fee, &sender);
    assert_eq!(res, 1);
    assert_eq!(
        ctx.env.auths(),
        std::vec![
            (
                sender.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        client.address.clone(),
                        symbol_short!("send_call"),
                        (envelope, ctx.network_address.clone(), sent_fee, &sender,)
                            .into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            ctx.native_token.clone(),
                            symbol_short!("transfer"),
                            (sender.clone(), ctx.contract.clone(), sent_fee as i128)
                                .into_val(&ctx.env)
                        )),
                        sub_invocations: std::vec![]
                    }]
                }
            ),
            (
                ctx.contract.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.native_token.clone(),
                        symbol_short!("transfer"),
                        (
                            &ctx.contract.clone(),
                            ctx.centralized_connection.clone(),
                            connection_fee.clone() as i128
                        )
                            .into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            )
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        let sn = Xcall::get_next_sn(&ctx.env);
        assert_eq!(sn, res + 1)
    });

    let fee_handler_balance = ctx.get_native_token_balance(&ctx.admin);
    let expected_balance = (sent_fee - fee) + protocol_fee;
    assert_eq!(fee_handler_balance, expected_balance);

    let xcall_balance = ctx.get_native_token_balance(&ctx.contract);
    assert_eq!(xcall_balance, 0);

    let sender_balance = ctx.get_native_token_balance(&sender);
    assert_eq!(sender_balance, mint_amount - sent_fee);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_message_with_greater_than_max_data_size() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let mut bytes = Bytes::new(&ctx.env);
    bytes.copy_from_slice(2050, &[1; 2050]);

    let msg = AnyMessage::CallMessage(CallMessage { data: bytes });
    let envelope = get_dummy_envelope_msg(&ctx.env, msg);
    client.send_call(&envelope, &ctx.network_address, &100, &ctx.admin);
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

    Xcall::process_message(&ctx.env, &ctx.network_address, 1, &sender, envelope).unwrap();
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

    Xcall::process_message(&ctx.env, &ctx.network_address, 1, &ctx.contract, envelope).unwrap();
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
        let res =
            Xcall::process_message(&ctx.env, &ctx.network_address, 1, &ctx.contract, envelope);
        let rollback = Xcall::get_rollback(&ctx.env, 1);
        assert!(rollback.is_ok());
        assert!(res.is_ok())
    });
}

#[test]
fn test_call_connection_for_rollback_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg = Bytes::new(&ctx.env);
    let sources = vec![&ctx.env, ctx.centralized_connection.to_string()];

    let need_response = true;
    let fee = ctx.get_centralized_connection_fee(need_response);
    // temporary mint to test call_connection
    ctx.mint_native_token(&ctx.contract, fee);
    let res = Xcall::call_connection(
        &ctx.env,
        &ctx.nid,
        1,
        sources.clone(),
        need_response,
        msg.clone(),
    )
    .unwrap();
    assert_eq!(res, fee);

    let xcall_balance = ctx.get_native_token_balance(&ctx.contract);
    let connection_balance = ctx.get_native_token_balance(&ctx.centralized_connection);

    assert_eq!(xcall_balance, 0);
    assert_eq!(connection_balance, fee);
}

#[test]
fn test_call_connection_for_call_message() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg = Bytes::new(&ctx.env);
    let sources = vec![&ctx.env, ctx.centralized_connection.to_string()];

    let need_response = false;
    let fee = ctx.get_centralized_connection_fee(need_response);
    // temporary mint to test call_connection
    ctx.mint_native_token(&ctx.contract, fee);
    let res = Xcall::call_connection(
        &ctx.env,
        &ctx.nid,
        1,
        sources.clone(),
        need_response,
        msg.clone(),
    )
    .unwrap();
    assert_eq!(res, fee);

    let xcall_balance = ctx.get_native_token_balance(&ctx.contract);
    let connection_balance = ctx.get_native_token_balance(&ctx.centralized_connection);

    assert_eq!(xcall_balance, 0);
    assert_eq!(connection_balance, fee);
}

#[test]
fn test_calim_protocol_fee() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sent_amount = 300;
    let connections_fee = 200;
    let remaining_fee = sent_amount - connections_fee;

    // temporary mint to test claim_protocol_fee
    ctx.mint_native_token(&ctx.contract, 100);

    ctx.env.as_contract(&ctx.contract, || {
        Xcall::claim_protocol_fee(&ctx.env, sent_amount, connections_fee).unwrap();

        let fee_handler_balance = ctx.get_native_token_balance(&ctx.admin);
        let xcall_balance = ctx.get_native_token_balance(&ctx.contract);

        assert_eq!(fee_handler_balance, remaining_fee);
        assert_eq!(xcall_balance, 0);
    });
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn test_claim_protocol_fail_for_insufficient_amount_sent() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sent_amount = 300;
    let connections_fee = 200;

    // temporary mint to test claim_protocol_fee
    ctx.mint_native_token(&ctx.contract, 100);

    ctx.env.as_contract(&ctx.contract, || {
        Xcall::set_protocol_fee(&ctx.env, 150).unwrap();
        Xcall::claim_protocol_fee(&ctx.env, sent_amount, connections_fee).unwrap();

        let xcall_balance = ctx.get_native_token_balance(&ctx.contract);
        assert_eq!(xcall_balance, 0);
    });
}

#[test]
fn test_array_equal_for_mismatch_length() {
    let ctx = TestContext::default();

    let protocols = get_dummy_sources(&ctx.env);
    let sources: Vec<String> = vec![&ctx.env];

    ctx.env.as_contract(&ctx.contract, || {
        let res = Xcall::are_array_equal(&protocols, &sources);
        assert_eq!(res, false)
    });
}

#[test]
fn test_array_equal_returns_false_for_unknown_protocol() {
    let ctx = TestContext::default();

    let protocols = get_dummy_sources(&ctx.env);
    let sources: Vec<String> = vec![
        &ctx.env,
        String::from_str(&ctx.env, "layerzero"),
        String::from_str(&ctx.env, "wormhole"),
    ];

    ctx.env.as_contract(&ctx.contract, || {
        let res = Xcall::are_array_equal(&protocols, &sources);
        assert_eq!(res, false)
    });
}

#[test]
fn test_array_equal() {
    let ctx = TestContext::default();

    let protocols = get_dummy_sources(&ctx.env);
    let sources = get_dummy_destinations(&ctx.env);

    ctx.env.as_contract(&ctx.contract, || {
        let res = Xcall::are_array_equal(&protocols, &sources);
        assert_eq!(res, true)
    });
}

#[test]
fn test_is_reply_for_mismatch_network() {
    let ctx = TestContext::default();

    ctx.env.as_contract(&ctx.contract, || {
        let req = get_dummy_message_request(&ctx.env);
        Xcall::store_reply_state(&ctx.env, &req);

        let nid = String::from_str(&ctx.env, "icon");
        let res = Xcall::is_reply(&ctx.env, &nid, req.protocols());
        assert_eq!(res, false)
    });
}

#[test]
fn test_is_reply_returns_false_when_missing_reply_state() {
    let ctx = TestContext::default();

    let sources = get_dummy_sources(&ctx.env);
    let nid = String::from_str(&ctx.env, "icon");

    ctx.env.as_contract(&ctx.contract, || {
        let res = Xcall::is_reply(&ctx.env, &nid, &sources);
        assert_eq!(res, false)
    });
}
