#![cfg(test)]

use soroban_rlp::encoder;
use soroban_sdk::{
    bytes,
    testutils::{Address as _, Events},
    vec, Address, Bytes, IntoVal, String, Vec,
};
use soroban_xcall_lib::{messages::msg_type::MessageType, network_address::NetworkAddress};

use crate::{
    contract::XcallClient,
    event::{CallExecutedEvent, RollbackExecutedEvent},
    storage,
    types::{request::CSMessageRequest, rollback::Rollback},
};

use super::setup::*;

#[test]
fn test_execute_call_with_persistent_message_type() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let req_id = 1;
    let sequence_no = 1;
    let from = NetworkAddress::new(&ctx.env, ctx.nid, ctx.contract.to_string());
    let mut req = CSMessageRequest::new(
        from,
        ctx.dapp.to_string(),
        sequence_no,
        get_dummy_protocols(&ctx.env),
        MessageType::CallMessagePersisted,
        Bytes::new(&ctx.env),
    );
    req.hash_data(&ctx.env);

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_proxy_request(&ctx.env, req_id.clone(), &req);
    });

    client.execute_call(&ctx.admin, &req_id, &Bytes::new(&ctx.env));

    let call_executed_event = CallExecutedEvent {
        reqId: req_id,
        code: 1,
        msg: String::from_str(&ctx.env, "success"),
    };
    let events = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("CallExecuted",).into_val(&ctx.env),
                call_executed_event.into_val(&ctx.env)
            ),
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        // request should be removed
        assert!(storage::get_proxy_request(&ctx.env, req_id).is_err());
    });
}

#[test]
fn test_execute_call_with_call_message_type() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg_data = encoder::encode_string(&ctx.env, String::from_str(&ctx.env, "rollback"));

    let req_id = 1;
    let sequence_no = 1;
    let mut req = CSMessageRequest::new(
        ctx.network_address,
        ctx.dapp.to_string(),
        sequence_no,
        get_dummy_protocols(&ctx.env),
        MessageType::CallMessage,
        msg_data.clone(),
    );
    req.hash_data(&ctx.env);

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_proxy_request(&ctx.env, req_id.clone(), &req);
    });

    client.execute_call(&ctx.admin, &req_id, &msg_data);

    let call_executed_event = CallExecutedEvent {
        reqId: req_id,
        code: 0,
        msg: String::from_str(&ctx.env, "unknown error"),
    };
    let events = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("CallExecuted",).into_val(&ctx.env),
                call_executed_event.into_val(&ctx.env)
            ),
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        // request should be removed
        assert!(storage::get_proxy_request(&ctx.env, req_id).is_err());
    });
}

#[test]
fn test_execute_call_with_rollback_message_type() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg_data = encoder::encode_string(&ctx.env, String::from_str(&ctx.env, "abc"));

    let req_id = 1;
    let sequence_no = 1;
    let mut req = CSMessageRequest::new(
        ctx.network_address,
        ctx.dapp.to_string(),
        sequence_no,
        Vec::new(&ctx.env),
        MessageType::CallMessageWithRollback,
        msg_data.clone(),
    );
    req.hash_data(&ctx.env);

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_proxy_request(&ctx.env, req_id.clone(), &req);
    });

    client.execute_call(&ctx.admin, &req_id, &msg_data);

    let call_executed_event = CallExecutedEvent {
        reqId: req_id,
        code: 1,
        msg: String::from_str(&ctx.env, "success"),
    };
    let events = vec![&ctx.env, ctx.env.events().all().get(1).unwrap()];
    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("CallExecuted",).into_val(&ctx.env),
                call_executed_event.into_val(&ctx.env)
            ),
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        // request should be removed
        assert!(storage::get_proxy_request(&ctx.env, req_id).is_err());
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_execute_call_data_mismatch() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let req_id = 1;
    let req = get_dummy_message_request(&ctx.env);

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_proxy_request(&ctx.env, req_id.clone(), &req);
    });

    client.execute_call(&ctx.admin, &req_id, &Bytes::new(&ctx.env));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #14)")]
fn test_execute_rollback_fail_for_invalid_sequence_number() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    let sequence_no = 1;
    client.execute_rollback(&sequence_no);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_execute_rollback_fail_not_enabled() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    let sequence_no = 1;
    let rollback = Rollback::new(
        Address::generate(&ctx.env),
        ctx.network_address,
        get_dummy_sources(&ctx.env),
        bytes!(&ctx.env, 0xabc),
        false,
    );

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    client.execute_rollback(&sequence_no);
}

#[test]
fn test_execute_rollback_success() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let sequence_no = 1;
    let rollback = Rollback::new(
        ctx.dapp,
        ctx.network_address,
        get_dummy_sources(&ctx.env),
        Bytes::new(&ctx.env),
        true,
    );

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    client.execute_rollback(&sequence_no);

    let rollback_executed_event = RollbackExecutedEvent { sn: sequence_no };
    let events = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        events,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("RollbackExecuted",).into_val(&ctx.env),
                rollback_executed_event.into_val(&ctx.env)
            ),
        ]
    );

    ctx.env.as_contract(&ctx.contract, || {
        // rollback should be removed
        assert!(storage::get_rollback(&ctx.env, sequence_no).is_err());
    });
}
