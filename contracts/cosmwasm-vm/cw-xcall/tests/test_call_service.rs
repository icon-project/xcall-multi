mod account;
mod setup;

use common::utils::keccak256;
use cw2::{get_contract_version, ContractVersion};
use cw_xcall::MigrateMsg;
use setup::{test::*, *};
use std::str::FromStr;

use cosmwasm_std::{
    testing::{mock_env, MOCK_CONTRACT_ADDR},
    to_json_binary, Addr, Event, Reply, SubMsgResponse, SubMsgResult,
};
use cw_xcall::{
    execute, instantiate, migrate,
    msg::{InstantiateMsg, QueryMsg},
    query, reply,
    state::CwCallService,
    types::{request::CSMessageRequest, rollback::Rollback},
};
use cw_xcall_lib::{
    message::msg_type::MessageType,
    network_address::{NetId, NetworkAddress},
    xcall_msg::ExecuteMsg,
};

#[test]
fn proper_instantiate() {
    let mut mock_deps = deps();
    let mock_info = create_mock_info(MOCK_CONTRACT_ADDR, "umlg", 2000);
    let env = mock_env();
    let store = CwCallService::default();

    let res = instantiate(
        mock_deps.as_mut(),
        env,
        mock_info,
        InstantiateMsg {
            network_id: "nid".to_string(),
            denom: "arch".to_string(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 0);

    let last_request_id = store
        .query_last_request_id(mock_deps.as_ref().storage)
        .unwrap();

    assert_eq!(0, last_request_id);

    let admin = store.query_admin(mock_deps.as_ref().storage).unwrap();

    assert_eq!(MOCK_CONTRACT_ADDR, admin)
}

#[test]
#[should_panic(expected = "NotFound")]
fn improper_instantiate() {
    let mock_deps = deps();

    let store = CwCallService::default();

    let last_request_id = store
        .query_last_request_id(mock_deps.as_ref().storage)
        .unwrap();

    assert_eq!(0, last_request_id);
}

#[test]
fn test_migrate() {
    let ctx = TestContext::default();
    let mut deps = deps();

    const CONTRACT_NAME: &str = "crates.io:cw-xcall";
    const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

    migrate(deps.as_mut(), ctx.env, MigrateMsg {}).unwrap();
    let expected = ContractVersion {
        contract: CONTRACT_NAME.to_string(),
        version: CONTRACT_VERSION.to_string(),
    };
    let version = get_contract_version(deps.as_ref().storage).unwrap();
    assert_eq!(expected, version);
}

#[test]
#[should_panic(expected = "ReplyError { code: 5, msg: \"Unknown\" }")]
fn test_reply_fail_for_unknown_reply_id() {
    let mut deps = deps();
    let ctx = TestContext::default();

    let msg = Reply {
        id: 5,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![Event::new("empty")],
            data: Some(to_json_binary(&vec![1]).unwrap()),
        }),
    };
    reply(deps.as_mut(), ctx.env, msg).unwrap();
}

#[test]
fn test_execute_set_admin() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = ExecuteMsg::SetAdmin {
        address: "new".to_string(),
    };
    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg);
    assert!(res.is_ok())
}

#[test]
fn test_execute_set_protocol_fee_handler() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = ExecuteMsg::SetProtocolFeeHandler {
        address: "fee_handler".to_string(),
    };
    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg).unwrap();

    assert_eq!(res.attributes[0].value, "set_protocol_feehandler");
}

#[test]
fn test_execute_set_default_connection() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = ExecuteMsg::SetDefaultConnection {
        nid: ctx.nid,
        address: Addr::unchecked("icon_contract"),
    };
    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg);
    assert!(res.is_ok())
}

#[test]
fn test_execute_send_call_message() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = ExecuteMsg::SendCallMessage {
        to: get_dummy_network_address("archway"),
        data: vec![1, 2, 3],
        rollback: None,
        sources: Some(vec![]),
        destinations: Some(vec![]),
    };

    mock_connection_fee_query(&mut deps);

    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "xcall-service");
    assert_eq!(res.attributes[1].value, "send_packet");
    assert_eq!(res.attributes[2].value, "1");
}

#[test]
fn test_execute_send_call() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    mock_connection_fee_query(&mut deps);

    let envelope = get_dummy_call_msg_envelop();
    let msg = ExecuteMsg::SendCall {
        envelope,
        to: get_dummy_network_address("archway"),
    };
    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "xcall-service");
    assert_eq!(res.attributes[1].value, "send_packet");
    assert_eq!(res.attributes[2].value, "1");
}

#[test]
fn test_execute_handle_request_message_with_default_connection() {
    let mut deps = deps();
    let contract = CwCallService::new();
    let info = create_mock_info("centralized", "icx", 100);

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let from_nid = NetId::from_str("archway").unwrap();
    let msg = ExecuteMsg::HandleMessage {
        from_nid,
        msg: get_dummy_request_message().as_bytes(),
    };
    let res = execute(deps.as_mut(), ctx.env, info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "call_service");
    assert_eq!(res.attributes[1].value, "handle_response")
}

#[test]
fn test_execute_call() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_execute_call(deps.as_mut().storage, &contract);

    let msg = ExecuteMsg::ExecuteCall {
        request_id: ctx.request_id,
        data: vec![1, 2, 3],
    };
    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg).unwrap();
    assert_eq!(res.attributes[1].value, "execute_call")
}

#[test]
fn test_execute_handle_error() {
    let mut deps = deps();
    let contract = CwCallService::new();
    let info = create_mock_info("centralized", "arch", 100);

    let ctx = TestContext::default();
    ctx.init_execute_call(deps.as_mut().storage, &contract);

    let rollback = get_dummy_rollback_data();
    contract
        .store_call_request(deps.as_mut().storage, ctx.request_id, &rollback)
        .unwrap();

    let msg = ExecuteMsg::HandleError { sn: 0 };
    let res = execute(deps.as_mut(), ctx.env, info, msg).unwrap();
    assert_eq!(res.attributes[1].value, "handle_response")
}

#[test]
fn test_execute_rollback() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_execute_call(deps.as_mut().storage, &contract);

    let rollback = Rollback::new(
        Addr::unchecked("dapp"),
        get_dummy_network_address("archway"),
        vec!["src".to_string()],
        vec![1, 2, 3],
        true,
    );
    contract
        .store_call_request(deps.as_mut().storage, ctx.request_id, &rollback)
        .unwrap();

    let msg = ExecuteMsg::ExecuteRollback {
        sequence_no: ctx.request_id,
    };
    let res = execute(deps.as_mut(), ctx.env, ctx.info, msg).unwrap();
    assert_eq!(res.attributes[1].value, "execute_rollback");
}

#[test]
fn test_query_get_admin() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = QueryMsg::GetAdmin {};
    let res = query(deps.as_ref(), ctx.env, msg).unwrap();
    assert_eq!(res, to_json_binary("admin").unwrap())
}

#[test]
fn test_query_get_network_address() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = QueryMsg::GetNetworkAddress {};
    let res = query(deps.as_ref(), ctx.env.clone(), msg).unwrap();

    let expected_network_address =
        NetworkAddress::new("icon", ctx.env.contract.address.clone().as_str());
    assert_eq!(res, to_json_binary(&expected_network_address).unwrap())
}

#[test]
fn test_query_verify_success() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    ctx.set_successful_response(deps.as_mut().storage, &contract, 1);

    let msg = QueryMsg::VerifySuccess { sn: 1 };
    let res = query(deps.as_ref(), ctx.env, msg);
    assert!(res.is_ok())
}

#[test]
fn test_query_get_default_connection() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let msg = QueryMsg::GetDefaultConnection {
        nid: NetId::from_str("archway").unwrap(),
    };
    let res = query(deps.as_ref(), ctx.env, msg).unwrap();
    assert_eq!(
        res,
        to_json_binary(&Addr::unchecked("centralized")).unwrap()
    );
}

#[test]
fn test_get_all_connection() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let res = contract.get_all_connections(deps.as_ref().storage).unwrap();
    assert_eq!(res, vec!["centralized".to_string()]);
}

#[test]
fn test_execute_call_reply() {
    let mut deps = deps();
    let contract = CwCallService::new();

    let ctx = TestContext::default();
    ctx.init_context(deps.as_mut().storage, &contract);

    let request: CSMessageRequest = CSMessageRequest::new(
        get_dummy_network_address("archway"),
        Addr::unchecked("xcall"),
        u128::default(),
        MessageType::CallMessageWithRollback,
        keccak256(&[1, 2, 3]).to_vec(),
        vec![],
    );
    contract
        .store_proxy_request(deps.as_mut().storage, ctx.request_id, &request)
        .unwrap();
    contract
        .save_call_reply(deps.as_mut().storage, &ctx.request_message.unwrap())
        .unwrap();
    contract
        .store_execute_request_id(deps.as_mut().storage, ctx.request_id)
        .unwrap();

    let msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![Event::new("empty")],
            data: Some(to_json_binary(&vec![1]).unwrap()),
        }),
    };

    let res = contract
        .execute_call_reply(deps.as_mut(), ctx.env, msg)
        .unwrap();
    assert_eq!(res.attributes[1].value, "execute_callback")
}
