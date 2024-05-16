#![cfg(test)]

use soroban_sdk::{
    bytes,
    testutils::{Address as _, Events},
    vec, Address, IntoVal, String,
};
use soroban_xcall_lib::messages::msg_type::MessageType;

use crate::{
    contract::{Xcall, XcallClient},
    event::CallMsgEvent,
    types::{
        message::CSMessage,
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
        rollback::Rollback,
    },
};

use super::setup::*;

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_handle_message_fail_for_same_network_id() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    client.handle_message(
        &Address::generate(&ctx.env),
        &String::from_str(&ctx.env, "icon"),
        &bytes!(&ctx.env),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_handle_message_request_fail_for_invalid_network_id() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let request = CSMessageRequest::new(
        ctx.network_address,
        Address::generate(&ctx.env).to_string(),
        1,
        vec![&ctx.env],
        MessageType::CallMessage,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_request(&ctx.env, &request).encode(&ctx.env);

    client.handle_message(
        &ctx.centralized_connection,
        &String::from_str(&ctx.env, "cosmos"),
        &cs_message,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_handle_message_request_fail_for_invalid_source() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let request = CSMessageRequest::new(
        ctx.network_address,
        Address::generate(&ctx.env).to_string(),
        1,
        vec![&ctx.env],
        MessageType::CallMessage,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_request(&ctx.env, &request).encode(&ctx.env);

    client.handle_message(
        &Address::generate(&ctx.env),
        &String::from_str(&ctx.env, "stellar"),
        &cs_message,
    );
}

#[test]
fn test_handle_message_request_from_default_connection() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let request = CSMessageRequest::new(
        ctx.network_address,
        Address::generate(&ctx.env).to_string(),
        1,
        vec![&ctx.env],
        MessageType::CallMessage,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_request(&ctx.env, &request).encode(&ctx.env);

    let res = client.handle_message(
        &ctx.centralized_connection,
        &String::from_str(&ctx.env, "stellar"),
        &cs_message,
    );
    assert_eq!(res, ());

    ctx.env.as_contract(&ctx.contract, || {
        let last_request_id = Xcall::increment_last_request_id(&ctx.env);
        assert_eq!(last_request_id, 2);

        let proxy_request = Xcall::get_proxy_request(&ctx.env, 1).unwrap();
        assert_eq!(proxy_request.sequence_no(), 1);
        assert_eq!(proxy_request.from(), request.from());
        assert_eq!(proxy_request.msg_type(), request.msg_type());
    })
}

#[test]
fn test_handle_message_request_from_multiple_sources() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let protocols = get_dummy_protocols(&ctx.env);
    let request = CSMessageRequest::new(
        ctx.network_address,
        Address::generate(&ctx.env).to_string(),
        1,
        protocols.clone(),
        MessageType::CallMessagePersisted,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_request(&ctx.env, &request);
    let encoded = cs_message.encode(&ctx.env);

    for (i, protocol) in protocols.iter().enumerate() {
        let from_nid = String::from_str(&ctx.env, "stellar");
        let sender = Address::from_string(&protocol);

        let res = client.handle_message(&sender, &from_nid, &encoded);
        assert_eq!(res, ());

        let cs_message = CSMessage::decode(&ctx.env, encoded.clone()).unwrap();
        let hash = ctx.env.crypto().keccak256(cs_message.payload());

        ctx.env.as_contract(&ctx.contract, || {
            let pending_requests = Xcall::get_pending_request(&ctx.env, hash);

            let i = i as u32 + 1;
            if i < protocols.len() {
                assert_eq!(pending_requests.len(), i)
            } else {
                assert_eq!(pending_requests.len(), 0)
            }
        })
    }

    let event_msg = CallMsgEvent {
        from: request.from().to_string(),
        to: request.to().clone(),
        sn: 1_u128,
        reqId: 1_u128,
        data: request.data().clone(),
    };
    let event = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        event,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("CallMessage",).into_val(&ctx.env),
                event_msg.into_val(&ctx.env)
            )
        ]
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #14)")]
fn test_handle_message_result_fail_for_invalid_sequence_no() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sequence_no = 10;
    let result = CSMessageResult::new(
        sequence_no,
        CSResponseType::CSResponseSuccess,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_result(&ctx.env, &result).encode(&ctx.env);

    client.handle_message(
        &ctx.centralized_connection,
        &String::from_str(&ctx.env, "cosmos"),
        &cs_message,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_handle_message_result_fail_for_invalid_sender() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sequence_no = 10;
    let rollback = Rollback::new(
        Address::generate(&ctx.env),
        ctx.network_address,
        get_dummy_sources(&ctx.env),
        bytes!(&ctx.env, 0xabc),
        false,
    );
    ctx.env.as_contract(&ctx.contract, || {
        Xcall::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    let result = CSMessageResult::new(
        sequence_no,
        CSResponseType::CSResponseSuccess,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_result(&ctx.env, &result).encode(&ctx.env);

    client.handle_message(
        &Address::generate(&ctx.env),
        &String::from_str(&ctx.env, "cosmos"),
        &cs_message,
    );
}
