mod setup;
use std::str::FromStr;

use anyhow::Error as AppError;

use common::rlp::Nullable;

use cosmwasm_std::Addr;
use cosmwasm_std::IbcChannel;

use cw_common::raw_types::channel::RawPacket;

use cw_multi_test::AppResponse;

use cw_multi_test::Executor;

use cw_xcall::types::message::CSMessage;
use cw_xcall::types::request::CSMessageRequest;
use cw_xcall::types::result::CSMessageResult;
use cw_xcall::types::result::CallServiceResponseType;
use cw_xcall_ibc_connection::types::message::Message;
use cw_xcall_lib::message::call_message_rollback::CallMessageWithRollback;
use cw_xcall_lib::message::envelope::Envelope;
use cw_xcall_lib::message::AnyMessage;

use cw_common::ProstMessage;
use cw_xcall_lib::message::msg_type::MessageType;
use cw_xcall_lib::network_address::NetworkAddress;
use setup::init_mock_dapp_multi_contract;
use setup::{
    init_mock_ibc_core_contract, init_xcall_app_contract, init_xcall_ibc_connection_contract,
    TestContext,
};
use xcall_lib::network_address::NetId;

use crate::setup::get_event;
use crate::setup::mock_ibc_config;
use crate::setup::setup_context;
const MOCK_CONTRACT_TO_ADDR: &str = "cosmoscontract";

fn setup_contracts(mut ctx: TestContext) -> TestContext {
    ctx = init_mock_ibc_core_contract(ctx);
    // ctx.set_ibc_core(ctx.sender.clone());
    ctx = init_xcall_app_contract(ctx);
    ctx = init_xcall_ibc_connection_contract(ctx);
    ctx = init_mock_dapp_multi_contract(ctx);
    ctx
}

fn setup_test() -> TestContext {
    let mut context = setup_context();
    context = setup_contracts(context);
    context
}

pub fn call_send_call_message(
    ctx: &mut TestContext,
    to: &str,
    sources: Vec<String>,
    destinations: Vec<String>,
    data: Vec<u8>,
    rollback: Option<Vec<u8>>,
) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_xcall_app(),
        &xcall_lib::xcall_msg::ExecuteMsg::SendCallMessage {
            to: xcall_lib::network_address::NetworkAddress::from_str(to).unwrap(),
            data,
            rollback,
            sources: Some(sources),
            destinations: Some(destinations),
        },
        &[],
    )
}

pub fn call_execute_call_message(
    ctx: &mut TestContext,
    request_id: u128,
    data: Vec<u8>,
) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_xcall_app(),
        &xcall_lib::xcall_msg::ExecuteMsg::ExecuteCall { request_id, data },
        &[],
    )
}

pub fn call_dapp_send_call(
    ctx: &mut TestContext,
    to: String,
    envelope: Envelope,
) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_dapp(),
        &cw_mock_dapp_multi::msg::ExecuteMsg::SendMessageAny {
            to: cw_xcall_lib::network_address::NetworkAddress::from_str(&to).unwrap(),
            envelope,
        },
        &[],
    )
}

pub fn call_dapp_add_connection(
    ctx: &mut TestContext,
    src_endpoint: String,
    dest_endpoint: String,
    network_id: String,
) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_dapp(),
        &cw_mock_dapp_multi::msg::ExecuteMsg::AddConnection {
            src_endpoint,
            dest_endpoint,
            network_id,
        },
        &[],
    )
}

pub fn call_set_xcall_host(ctx: &mut TestContext) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_xcall_ibc_connection(),
        &cw_common::xcall_connection_msg::ExecuteMsg::SetXCallHost {
            address: ctx.get_xcall_app().to_string(),
        },
        &[],
    )
}

pub fn call_set_default_connection(
    ctx: &mut TestContext,
    nid: String,
) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_xcall_app(),
        &xcall_lib::xcall_msg::ExecuteMsg::SetDefaultConnection {
            nid: NetId::from(nid),
            address: ctx.get_xcall_ibc_connection(),
        },
        &[],
    )
}

pub fn call_configure_connection(
    ctx: &mut TestContext,
    connection_id: String,
    nid: String,
    client_id: String,
) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_xcall_ibc_connection(),
        &cw_common::xcall_connection_msg::ExecuteMsg::ConfigureConnection {
            connection_id,
            counterparty_port_id: "xcall".to_string(),
            counterparty_nid: NetId::from_str(&nid).unwrap(),
            client_id,
            timeout_height: 10,
        },
        &[],
    )
}

pub fn call_ibc_channel_connect(ctx: &mut TestContext) -> Result<AppResponse, AppError> {
    let ibc_config = mock_ibc_config();
    let channel = IbcChannel::new(
        ibc_config.src_endpoint().clone(),
        ibc_config.dst_endpoint().clone(),
        cosmwasm_std::IbcOrder::Unordered,
        "ics-20",
        "connection-1",
    );

    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_ibc_core(),
        &cw_mock_ibc_core::msg::ExecuteMsg::IbcConfig {
            msg: cosmwasm_std::IbcChannelConnectMsg::OpenConfirm { channel },
        },
        &[],
    )
}

pub fn call_ibc_receive_packet(
    ctx: &mut TestContext,
    msg: Vec<u8>,
) -> Result<AppResponse, AppError> {
    let ibc_config = mock_ibc_config();
    let packet = RawPacket {
        sequence: 1,
        source_port: ibc_config.dst_endpoint().port_id.to_string(),
        source_channel: ibc_config.dst_endpoint().channel_id.to_string(),
        destination_port: ibc_config.src_endpoint().port_id.to_string(),
        destination_channel: ibc_config.src_endpoint().channel_id.to_string(),
        data: msg,
        timeout_height: Some(cw_common::raw_types::RawHeight {
            revision_number: 0,
            revision_height: 12345,
        }),
        timeout_timestamp: 17747483838282,
    };
    let packet_bytes = hex::encode(packet.encode_to_vec());
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_ibc_core(),
        &cw_mock_ibc_core::msg::ExecuteMsg::ReceivePacket {
            message: packet_bytes,
        },
        &[],
    )
}

pub fn call_register_connection(ctx: &mut TestContext) -> Result<AppResponse, AppError> {
    ctx.app.execute_contract(
        ctx.sender.clone(),
        ctx.get_ibc_core(),
        &cw_mock_ibc_core::msg::ExecuteMsg::RegisterXcall {
            address: ctx.get_xcall_ibc_connection(),
        },
        &[],
    )
}

#[test]
fn test_xcall_send_call_message() {
    let mut ctx = setup_test();
    call_set_xcall_host(&mut ctx).unwrap();
    call_register_connection(&mut ctx).unwrap();
    let src = ctx.get_xcall_ibc_connection().to_string();

    let nid = "0x3.icon";
    call_configure_connection(
        &mut ctx,
        "connection-1".to_string(),
        nid.to_string(),
        "client-1".to_string(),
    )
    .unwrap();
    call_ibc_channel_connect(&mut ctx).unwrap();
    let result = call_send_call_message(
        &mut ctx,
        &format!("{nid}/{MOCK_CONTRACT_TO_ADDR}"),
        vec![src],
        vec!["somedestination".to_string()],
        vec![1, 2, 3],
        None,
    );
    println!("{result:?}");
    assert!(result.is_ok());
    let result = result.unwrap();
    let event = get_event(&result, "wasm-CallMessageSent").unwrap();
    println!("{event:?}");
    assert_eq!(
        &format!("{nid}/{MOCK_CONTRACT_TO_ADDR}"),
        event.get("to").unwrap()
    );
}

#[test]
fn test_xcall_send_call() {
    let mut ctx = setup_test();
    call_set_xcall_host(&mut ctx).unwrap();
    call_register_connection(&mut ctx).unwrap();
    let src = ctx.get_xcall_ibc_connection().to_string();
    let dapp = ctx.get_dapp().to_string();

    let nid = "0x3.icon";
    call_configure_connection(
        &mut ctx,
        "connection-1".to_string(),
        nid.to_string(),
        "client-1".to_string(),
    )
    .unwrap();
    call_ibc_channel_connect(&mut ctx).unwrap();
    let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
        data: vec![1, 2, 3],
        rollback: "rollback-reply".as_bytes().to_vec(),
    });
    let envelope = Envelope::new(message, vec![src], vec!["somedestination".to_string()]);
    let result = call_dapp_send_call(&mut ctx, format!("{nid}/{dapp}"), envelope);
    println!("{result:?}");
    assert!(result.is_ok());
    let result = result.unwrap();
    let event = get_event(&result, "wasm-CallMessageSent").unwrap();
    println!("{event:?}");
    assert_eq!(&format!("{nid}/{dapp}"), event.get("to").unwrap());
}

#[test]
fn test_rollback_reply() {
    let mut ctx = setup_test();
    call_set_xcall_host(&mut ctx).unwrap();
    call_register_connection(&mut ctx).unwrap();
    let src = ctx.get_xcall_ibc_connection().to_string();
    let _dapp = ctx.get_dapp().to_string();

    let nid = "0x3.icon";
    call_configure_connection(
        &mut ctx,
        "connection-1".to_string(),
        nid.to_string(),
        "client-1".to_string(),
    )
    .unwrap();
    call_ibc_channel_connect(&mut ctx).unwrap();
    call_dapp_add_connection(&mut ctx, src, "somedest".to_string(), nid.to_string()).unwrap();
    let data = "reply-response".as_bytes().to_vec();
    let msg = CSMessageRequest::new(
        NetworkAddress::from_str(&format!("{nid}/{MOCK_CONTRACT_TO_ADDR}")).unwrap(),
        ctx.get_dapp(),
        1,
        MessageType::CallMessageWithRollback,
        data.clone(),
        vec![ctx.get_xcall_ibc_connection().to_string()],
    );
    let request = CSMessage {
        message_type: cw_xcall::types::message::CSMessageType::CSMessageRequest,
        payload: msg.as_bytes(),
    };

    let msg = Message {
        sn: Nullable::new(Some(1_i64)),
        fee: 0_u128,
        data: request.as_bytes(),
    };
    let bytes: Vec<u8> = common::rlp::encode(&msg).to_vec();

    call_ibc_receive_packet(&mut ctx, bytes).unwrap();
    let expected_reply = CSMessageRequest::new(
        NetworkAddress::from_str("nid/contract3").unwrap(),
        Addr::unchecked(MOCK_CONTRACT_TO_ADDR),
        1,
        MessageType::CallMessage,
        vec![1, 2, 3],
        vec!["somedest".to_string()],
    );
    let reply_message = CSMessageResult::new(
        expected_reply.sequence_no(),
        CallServiceResponseType::CallServiceResponseSuccess,
        Some(expected_reply.as_bytes()),
    );
    let message: CSMessage = reply_message.into();
    let expected_hex = hex::encode(message.as_bytes());

    let result = call_execute_call_message(&mut ctx, 1, data);
    println!("{result:?}");
    assert!(result.is_ok());
    let result = result.unwrap();
    let event = get_event(&result, "wasm-write_acknowledgement").unwrap();
    println!("{event:?}");
    assert_eq!(&expected_hex, event.get("data").unwrap());
}

fn test_call_message(
    ctx: &mut TestContext,
    data: Vec<u8>,
    msg_type: MessageType,
) -> Result<AppResponse, AppError> {
    call_set_xcall_host(ctx).unwrap();
    call_register_connection(ctx).unwrap();
    let src = ctx.get_xcall_ibc_connection().to_string();
    let _dapp = ctx.get_dapp().to_string();

    let nid = "0x3.icon";
    call_configure_connection(
        ctx,
        "connection-1".to_string(),
        nid.to_string(),
        "client-1".to_string(),
    )
    .unwrap();
    call_ibc_channel_connect(ctx).unwrap();
    call_dapp_add_connection(ctx, src, "somedest".to_string(), nid.to_string()).unwrap();
    let msg = CSMessageRequest::new(
        NetworkAddress::from_str(&format!("{nid}/{MOCK_CONTRACT_TO_ADDR}")).unwrap(),
        ctx.get_dapp(),
        1,
        msg_type,
        data.clone(),
        vec![ctx.get_xcall_ibc_connection().to_string()],
    );
    let request = CSMessage {
        message_type: cw_xcall::types::message::CSMessageType::CSMessageRequest,
        payload: msg.as_bytes(),
    };

    let msg = Message {
        sn: Nullable::new(Some(1_i64)),
        fee: 0_u128,
        data: request.as_bytes(),
    };
    let bytes: Vec<u8> = common::rlp::encode(&msg).to_vec();

    call_ibc_receive_packet(ctx, bytes).unwrap();
    call_execute_call_message(ctx, 1, data)
}

#[test]
fn test_call_message_failed() {
    let mut ctx = setup_test();

    let data = "rollback".as_bytes().to_vec();
    let resp = test_call_message(&mut ctx, data, MessageType::CallMessage);
    assert!(resp.is_ok());

    let event = get_event(&resp.unwrap(), "wasm-CallExecuted").unwrap();
    let expected_code: u8 = CallServiceResponseType::CallServiceResponseFailure.into();
    assert_eq!(event.get("code").unwrap(), &expected_code.to_string());
}

#[test]
fn test_call_message_success() {
    let mut ctx = setup_test();

    let data = "test".as_bytes().to_vec();
    let resp = test_call_message(&mut ctx, data, MessageType::CallMessage);
    assert!(resp.is_ok());
    let result = resp.unwrap();
    let event = get_event(&result, "wasm-CallExecuted").unwrap();
    let ack_event = get_event(&result, "wasm-write_acknowledgement");
    assert!(ack_event.is_none());

    let expected_code: u8 = CallServiceResponseType::CallServiceResponseSuccess.into();
    assert_eq!(event.get("code").unwrap(), &expected_code.to_string());
}

#[test]
#[should_panic(expected = "NotFound { kind: \"cw_xcall::types::request::CSMessageRequest\"")]
fn test_call_message_re_execute() {
    let mut ctx = setup_test();

    let data = "rollback".as_bytes().to_vec();
    let resp = test_call_message(&mut ctx, data.clone(), MessageType::CallMessage);
    assert!(resp.is_ok());
    // CallRequest should have been removed even though call failed
    let _ = call_execute_call_message(&mut ctx, 1, data);
}

#[test]
fn test_persistent_call_message_success() {
    let mut ctx = setup_test();

    let data = "test".as_bytes().to_vec();
    let resp = test_call_message(&mut ctx, data, MessageType::CallMessagePersisted);
    assert!(resp.is_ok());

    let result = resp.unwrap();
    let event = get_event(&result, "wasm-CallExecuted").unwrap();
    let ack_event = get_event(&result, "wasm-write_acknowledgement");
    assert!(ack_event.is_none());

    let expected_code: u8 = CallServiceResponseType::CallServiceResponseSuccess.into();
    assert_eq!(event.get("code").unwrap(), &expected_code.to_string());
}

#[test]
#[should_panic(expected = "NotFound { kind: \"cw_xcall::types::request::CSMessageRequest\"")]
fn test_persistent_call_message_re_execute() {
    let mut ctx = setup_test();

    let data = "test".as_bytes().to_vec();
    let resp = test_call_message(&mut ctx, data.clone(), MessageType::CallMessagePersisted);
    assert!(resp.is_ok());

    let result = resp.unwrap();
    let event = get_event(&result, "wasm-CallExecuted").unwrap();

    let expected_code: u8 = CallServiceResponseType::CallServiceResponseSuccess.into();
    assert_eq!(event.get("code").unwrap(), &expected_code.to_string());

    // removed after a successful execution
    let _ = call_execute_call_message(&mut ctx, 1, data);
}

#[test]
fn test_persistent_call_message_retry() {
    let mut ctx = setup_test();

    let data = "rollback".as_bytes().to_vec();
    let resp = test_call_message(&mut ctx, data.clone(), MessageType::CallMessagePersisted);
    assert!(resp.is_err());

    // can retry
    let resp = call_execute_call_message(&mut ctx, 1, data);
    assert!(resp.is_err());
}
