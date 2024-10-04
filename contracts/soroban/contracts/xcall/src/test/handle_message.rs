#![cfg(test)]

use soroban_sdk::{
    bytes,
    testutils::{Address as _, Events},
    vec, Address, IntoVal, String,
};
use soroban_xcall_lib::messages::msg_type::MessageType;

use crate::{
    contract::XcallClient,
    event::{CallMsgEvent, ResponseMsgEvent, RollbackMsgEvent},
    storage,
    types::{
        message::CSMessage,
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
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
        let last_request_id = storage::increment_last_request_id(&ctx.env);
        assert_eq!(last_request_id, 2);

        let proxy_request = storage::get_proxy_request(&ctx.env, 1).unwrap();
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
            let pending_requests = storage::get_pending_request(&ctx.env, hash);

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
    let rollback = get_dummy_rollback(&ctx.env);
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_rollback(&ctx.env, sequence_no, &rollback);
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

#[test]
fn test_handle_message_result_should_enable_rollback_when_response_is_failure_from_dst_chain() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sequence_no = 10;
    let rollback = get_dummy_rollback(&ctx.env);
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    let result = CSMessageResult::new(
        sequence_no,
        CSResponseType::CSResponseFailure,
        bytes!(&ctx.env, 0xabc),
    );
    let cs_message = CSMessage::from_result(&ctx.env, &result).encode(&ctx.env);

    client.handle_message(
        &ctx.centralized_connection,
        &String::from_str(&ctx.env, "cosmos"),
        &cs_message,
    );

    ctx.env.as_contract(&ctx.contract, || {
        // should enable rollback
        let rollback = storage::get_rollback(&ctx.env, sequence_no).unwrap();
        assert_eq!(rollback.enabled, true);
    });

    let response_msg_event = ResponseMsgEvent {
        sn: sequence_no,
        code: 0,
    };
    let rollback_event = RollbackMsgEvent { sn: sequence_no };

    let mut events = ctx.env.events().all();
    events.pop_front();

    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("ResponseMessage",).into_val(&ctx.env),
                response_msg_event.into_val(&ctx.env)
            ),
            (
                client.address.clone(),
                ("RollbackMessage",).into_val(&ctx.env),
                rollback_event.into_val(&ctx.env)
            )
        ]
    )
}

#[test]
fn test_handle_message_result_when_response_is_success_from_dst_chain() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let req_id = 1;
    let sequence_no = 1;
    let rollback = get_dummy_rollback(&ctx.env);
    ctx.env.as_contract(&ctx.contract, || {
        storage::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    let request = CSMessageRequest::new(
        ctx.network_address,
        Address::generate(&ctx.env).to_string(),
        sequence_no,
        get_dummy_protocols(&ctx.env),
        MessageType::CallMessage,
        bytes!(&ctx.env, 0xabc),
    );
    let result = CSMessageResult::new(
        sequence_no,
        CSResponseType::CSResponseSuccess,
        request.encode(&ctx.env),
    );
    let cs_message = CSMessage::from_result(&ctx.env, &result).encode(&ctx.env);

    client.handle_message(
        &ctx.centralized_connection,
        &String::from_str(&ctx.env, "cosmos"),
        &cs_message,
    );

    ctx.env.as_contract(&ctx.contract, || {
        // rollback should be removed
        assert!(storage::get_rollback(&ctx.env, sequence_no).is_err());

        // should save as successfull response
        let success_res = storage::get_successful_response(&ctx.env, sequence_no);
        assert_eq!(success_res, true);

        // should set proxy request
        let proxy_request = storage::get_proxy_request(&ctx.env, req_id).unwrap();
        assert_eq!(proxy_request.protocols(), rollback.protocols());

        // should set proxy request data as keccak256 hash value
        let mut req = request.clone();
        req.hash_data(&ctx.env);
        assert_eq!(req.data(), proxy_request.data())
    });

    let response_msg_event = ResponseMsgEvent {
        sn: sequence_no,
        code: 1,
    };
    let call_msg_event = CallMsgEvent {
        from: request.from().to_string(),
        to: request.to().clone(),
        sn: request.sequence_no(),
        reqId: req_id,
        data: request.data().clone(),
    };

    let mut events = ctx.env.events().all();
    events.pop_front();

    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("ResponseMessage",).into_val(&ctx.env),
                response_msg_event.into_val(&ctx.env)
            ),
            (
                client.address.clone(),
                ("CallMessage",).into_val(&ctx.env),
                call_msg_event.into_val(&ctx.env)
            )
        ]
    )
}
