mod account;
mod setup;
use crate::account::alice;

use common::utils::keccak256;
use schemars::_serde_json::to_string;
use setup::test::*;
use setup::*;
use std::str::FromStr;

use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env, mock_info},
    to_json_binary, Addr, Coin, CosmosMsg, Reply, SubMsgResponse, SubMsgResult, WasmMsg,
};
use cw_xcall::{
    state::{CwCallService, EXECUTE_CALL_ID},
    types::{
        message::{CSMessage, CSMessageType},
        request::CSMessageRequest,
        rollback::Rollback,
    },
};
use cw_xcall_lib::{
    message::msg_type::MessageType,
    network_address::{NetId, NetworkAddress},
};

#[test]
#[should_panic(expected = "InvalidRequestId")]
fn test_execute_call_invalid_request_id() {
    let cw_callservice = CwCallService::new();

    let deps = mock_dependencies();

    cw_callservice
        .contains_proxy_request(&deps.storage, 123456)
        .unwrap();
}

#[test]
#[should_panic(expected = "DataMismatch")]
fn test_execute_call_with_wrong_data() {
    let mut deps = mock_dependencies();

    let info = mock_info("user1", &[Coin::new(1000, "ucosm")]);
    let cw_callservice = CwCallService::default();
    let data = vec![104, 101, 108, 108, 111];
    let request_id = 123456;
    let proxy_requests = CSMessageRequest::new(
        NetworkAddress::new("nid", "mockaddress"),
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"),
        123,
        MessageType::CallMessage,
        keccak256(&[104, 106, 108, 108, 111]).to_vec(),
        vec![],
    );
    cw_callservice
        .store_proxy_request(deps.as_mut().storage, request_id, &proxy_requests)
        .unwrap();

    cw_callservice
        .execute_call(deps.as_mut(), info, request_id, data)
        .unwrap();
}

#[test]
fn test_execute_call_having_request_id_without_rollback() {
    let mut deps = mock_dependencies();

    let info = mock_info("user1", &[Coin::new(1000, "ucosm")]);
    let cw_callservice = CwCallService::default();
    let data = vec![104, 101, 108, 108, 111];
    let request_id = 123456;
    let proxy_requests = CSMessageRequest::new(
        NetworkAddress::new("nid", "mockaddress"),
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"),
        123,
        MessageType::CallMessage,
        keccak256(&data).to_vec(),
        vec![],
    );
    cw_callservice
        .store_proxy_request(deps.as_mut().storage, request_id, &proxy_requests)
        .unwrap();

    let res = cw_callservice
        .execute_call(deps.as_mut(), info, request_id, data)
        .unwrap();
    match &res.messages[0].msg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) => {
            assert_eq!(
                contract_addr,
                "88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"
            );

            assert_eq!(
                "\"eyJoYW5kbGVfY2FsbF9tZXNzYWdlIjp7ImZyb20iOiJuaWQvbW9ja2FkZHJlc3MiLCJkYXRhIjpbMTA0LDEwMSwxMDgsMTA4LDExMV19fQ==\"",
                to_string(msg).unwrap()
            )
        }
        _ => {}
    }
}

#[test]
fn test_successful_reply_message() {
    let mut mock_deps = deps();

    let env = mock_env();

    let msg = Reply {
        id: EXECUTE_CALL_ID,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };

    let contract = CwCallService::default();

    let request_id = 123456;
    let proxy_requests = CSMessageRequest::new(
        NetworkAddress::new("nid", "mockaddress"),
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"),
        123,
        MessageType::CallMessage,
        vec![],
        vec![],
    );
    contract
        .store_proxy_request(mock_deps.as_mut().storage, request_id, &proxy_requests)
        .unwrap();

    contract
        .store_execute_request_id(mock_deps.as_mut().storage, request_id)
        .unwrap();

    let response = contract.reply(mock_deps.as_mut(), env, msg).unwrap();

    assert_eq!(response.events[0].attributes[1].value, 1.to_string());
}

#[test]
fn test_failed_reply_message() {
    let mut mock_deps = deps();

    let env = mock_env();

    let msg = Reply {
        id: EXECUTE_CALL_ID,
        result: SubMsgResult::Err("error message".into()),
    };

    let contract = CwCallService::default();

    let request_id = 123456;
    let proxy_requests = CSMessageRequest::new(
        NetworkAddress::new("nid", "mockaddress"),
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"),
        123,
        MessageType::CallMessage,
        vec![],
        vec![],
    );
    contract
        .store_proxy_request(mock_deps.as_mut().storage, request_id, &proxy_requests)
        .unwrap();

    contract
        .store_execute_request_id(mock_deps.as_mut().storage, request_id)
        .unwrap();

    let response = contract.reply(mock_deps.as_mut(), env, msg).unwrap();

    assert_eq!(response.events[0].attributes[1].value, "0".to_string());
}

#[test]
#[should_panic(expected = "td(NotFound { kind: \"cw_xcall::types::request::CSMessageRequest\" })")]
fn test_invalid_sequence_no() {
    let deps = mock_dependencies();
    let contract = CwCallService::new();
    contract
        .get_proxy_request(deps.as_ref().storage, 123456)
        .unwrap();
}

#[test]
fn execute_rollback_success() {
    let mut mock_deps = deps();

    let mock_info = create_mock_info(&alice().to_string(), "umlg", 2000);

    let env = mock_env();

    let contract = CwCallService::default();
    contract
        .instantiate(
            mock_deps.as_mut(),
            env,
            mock_info.clone(),
            cw_xcall::msg::InstantiateMsg {
                network_id: "nid".to_string(),
                denom: "arch".to_string(),
            },
        )
        .unwrap();

    let seq_id = 123456;

    let request = Rollback::new(
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f126e4"),
        NetworkAddress::new("nid", "mockaddress"),
        vec![],
        vec![1, 2, 3],
        true,
    );

    contract
        .store_call_request(mock_deps.as_mut().storage, seq_id, &request)
        .unwrap();

    let response = contract
        .execute_rollback(mock_deps.as_mut(), mock_env(), mock_info, seq_id)
        .unwrap();

    match response.messages[0].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: _,
            msg,
            funds: _,
        }) => {
            let data = String::from_utf8(msg.0).unwrap();
            assert_eq!(
                "{\"handle_call_message\":{\"from\":\"nid/cosmos2contract\",\"data\":[1,2,3]}}",
                data
            )
        }
        _ => todo!(),
    }
    assert_eq!(seq_id.to_string(), response.events[0].attributes[0].value)
}

#[test]
#[should_panic(expected = "RollbackNotEnabled")]
fn execute_rollback_failure() {
    let mut mock_deps = deps();

    let mock_info = create_mock_info(&alice().to_string(), "umlg", 2000);

    let contract = CwCallService::default();

    let seq_id = 123456;

    let request = Rollback::new(
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f126e4"),
        NetworkAddress::new("nid", "mockaddress"),
        vec![],
        vec![],
        false,
    );

    contract
        .store_call_request(mock_deps.as_mut().storage, seq_id, &request)
        .unwrap();

    let response = contract
        .execute_rollback(mock_deps.as_mut(), mock_env(), mock_info, seq_id)
        .unwrap();

    match response.messages[0].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: _,
            msg,
            funds: _,
        }) => {
            let r: Vec<u64> = from_json(msg).unwrap();

            assert_eq!(vec![1, 2, 3], r)
        }
        _ => todo!(),
    }
}

#[test]
fn test_persisted_message_not_removed_on_error() {
    let mut mock_deps = deps();

    let env = mock_env();

    let msg = Reply {
        id: EXECUTE_CALL_ID,
        result: SubMsgResult::Err("error message".into()),
    };

    let contract = CwCallService::default();

    let request_id = 123456;
    let proxy_requests = CSMessageRequest::new(
        NetworkAddress::new("nid", "mockaddress"),
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"),
        123,
        MessageType::CallMessagePersisted,
        vec![],
        vec![],
    );
    contract
        .store_proxy_request(mock_deps.as_mut().storage, request_id, &proxy_requests)
        .unwrap();

    contract
        .store_execute_request_id(mock_deps.as_mut().storage, request_id)
        .unwrap();

    let _response = contract.reply(mock_deps.as_mut(), env, msg);

    assert!(_response.is_err());
}

#[test]
fn test_persisted_message_removed_on_success() {
    let mut mock_deps = deps();

    let env = mock_env();

    let msg = Reply {
        id: EXECUTE_CALL_ID,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: to_json_binary(&1).ok(),
        }),
    };

    let contract = CwCallService::default();

    let request_id = 123456;
    let proxy_requests = CSMessageRequest::new(
        NetworkAddress::new("nid", "mockaddress"),
        Addr::unchecked("88bd05442686be0a5df7da33b6f1089ebfea3769b19dbb2477fe0cd6e0f123t7"),
        123,
        MessageType::CallMessagePersisted,
        vec![],
        vec![],
    );
    contract
        .store_proxy_request(mock_deps.as_mut().storage, request_id, &proxy_requests)
        .unwrap();

    contract
        .store_execute_request_id(mock_deps.as_mut().storage, request_id)
        .unwrap();

    let _response = contract.reply(mock_deps.as_mut(), env, msg).unwrap();

    let req = contract
        .get_proxy_request(mock_deps.as_ref().storage, request_id)
        .ok();
    assert_eq!(req, None);
}

#[test]
fn test_handle_reply() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let request = get_dummy_req_msg();
    let rollback = get_dummy_rollback_data();

    let res = contract.handle_reply(deps.as_mut(), rollback, request);
    assert!(res.is_ok())
}

#[test]
#[should_panic(expected = "InvalidReplyReceived")]
fn test_handle_reply_fail() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let request = CSMessageRequest::new(
        get_dummy_network_address("icon"),
        Addr::unchecked("dapp"),
        1,
        MessageType::CallMessage,
        keccak256(&[1, 2, 3]).to_vec(),
        vec![],
    );
    let rollback = get_dummy_rollback_data();

    contract
        .handle_reply(deps.as_mut(), rollback, request)
        .unwrap();
}

#[test]
#[should_panic(expected = "ProtocolsMismatch")]
fn test_handle_request_fail_on_mismatch_protocols_request() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let data = ctx.request_message.unwrap().as_bytes();
    let src_net = NetId::from_str("evm").unwrap();
    contract
        .handle_request(deps.as_mut(), ctx.info, src_net, &data)
        .unwrap();
}

#[test]
#[should_panic(expected = "ProtocolsMismatch")]
fn test_handle_request_fail_on_invalid_source() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let nid = NetId::from_str("archway").unwrap();
    let data = ctx.request_message.unwrap().as_bytes();
    contract
        .handle_request(deps.as_mut(), ctx.info, nid, &data)
        .unwrap();
}

#[test]
fn test_handle_request_from_multiple_protocols() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let request = CSMessageRequest::new(
        get_dummy_network_address("archway"),
        Addr::unchecked("dapp"),
        1,
        MessageType::CallMessage,
        keccak256(&[1, 2, 3]).to_vec(),
        vec!["centralized".to_string(), "ibc".to_string()],
    );

    let nid = NetId::from_str("archway").unwrap();
    for protocol in request.protocols() {
        let info = create_mock_info(protocol, "icx", 100);
        let res = contract
            .handle_request(deps.as_mut(), info, nid.clone(), &request.as_bytes())
            .unwrap();
        if protocol == "ibc" {
            assert_eq!(res.attributes[0].value, "call_service");
        } else {
            assert_eq!(res.attributes.len(), 0)
        }
    }
}

#[test]
#[should_panic(expected = "CallRequestNotFound { sn: 1 }")]
fn test_handle_call_message_fail_on_invalid_request() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let message_result = get_dummy_result_message();
    let msg = CSMessage::new(CSMessageType::CSMessageResult, message_result.as_bytes());

    let nid = NetId::from_str("archway").unwrap();
    contract
        .handle_message(deps.as_mut(), ctx.info, nid, msg.as_bytes())
        .unwrap();
}

#[test]
#[should_panic(expected = "ProtocolsMismatch")]
fn test_handle_result_fail_on_invalid_source() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let rollback = get_dummy_rollback_data();
    contract
        .store_call_request(deps.as_mut().storage, 1, &rollback)
        .unwrap();

    let msg = get_dummy_result_message().as_bytes();
    contract
        .handle_result(deps.as_mut(), ctx.info, &msg)
        .unwrap();
}

#[test]
fn test_handle_result_from_multiple_protocols() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();

    ctx.init_context(deps.as_mut().storage, &contract);

    let rollback = Rollback::new(
        Addr::unchecked("xcall"),
        get_dummy_network_address("archway"),
        vec!["centralized".to_string(), "ibc".to_string()],
        vec![1, 2, 3],
        false,
    );
    contract
        .store_call_request(deps.as_mut().storage, 1, &rollback)
        .unwrap();

    let msg = get_dummy_result_message().as_bytes();

    for protocol in rollback.protocols() {
        let info = create_mock_info(protocol, "arch", 100);
        let res = contract.handle_result(deps.as_mut(), info, &msg).unwrap();
        if protocol == "centralized" {
            assert_eq!(res.attributes.len(), 0);
        } else {
            assert_eq!(res.attributes[1].value, "handle_response")
        }
    }
}

#[test]
fn test_handle_result_on_error_response() {
    let ctx = TestContext::default();
    let mut deps = deps();
    let contract = CwCallService::new();
    let info = create_mock_info("centralized", "arch", 100);

    ctx.init_reply_state(deps.as_mut().storage, &contract);

    let rollback = get_dummy_rollback_data();
    contract
        .store_call_request(deps.as_mut().storage, 1, &rollback)
        .unwrap();

    let msg = get_dummy_result_message_failure().as_bytes();
    let res = contract.handle_result(deps.as_mut(), info, &msg).unwrap();
    assert_eq!(res.attributes[1].value, "handle_response")
}
