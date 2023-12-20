mod setup;
use std::str::FromStr;

use anyhow::Error as AppError;

use cosmwasm_std::Addr;
use cosmwasm_std::IbcChannel;
use cosmwasm_std::IbcEndpoint;
use cw_multi_test::AppResponse;
use cw_multi_test::Executor;

use cw_xcall_lib::network_address::NetId;
use cw_xcall_lib::network_address::NetworkAddress;
use setup::{
    init_mock_ibc_core_contract, init_xcall_app_contract, init_xcall_ibc_connection_contract,
    TestContext,
};

use crate::setup::get_event;
use crate::setup::mock_ibc_config;
use crate::setup::setup_context;
const MOCK_CONTRACT_TO_ADDR: &str = "cosmoscontract";

fn setup_contracts(mut ctx: TestContext) -> TestContext {
    ctx = init_mock_ibc_core_contract(ctx);
   // ctx.set_ibc_core(ctx.sender.clone());
    ctx = init_xcall_app_contract(ctx);
    ctx = init_xcall_ibc_connection_contract(ctx);
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
        &cw_xcall_lib::xcall_msg::ExecuteMsg::SendCallMessage {
            to: NetworkAddress::from_str(to).unwrap(),
            data,
            rollback,
            sources: Some(sources),
            destinations: Some(destinations),
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
        &cw_xcall_lib::xcall_msg::ExecuteMsg::SetDefaultConnection {
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



pub fn call_channel_connect(ctx: &mut TestContext)->Result<AppResponse, AppError> {
    let ibc_config=mock_ibc_config();
    let channel= IbcChannel::new(
        ibc_config.src_endpoint().clone(),
        ibc_config.dst_endpoint().clone(),
        cosmwasm_std::IbcOrder::Unordered,
        "ics-20",
        "connection-1");

    ctx.app.execute_contract(ctx.sender.clone(), ctx.get_ibc_core(), 
    &cw_mock_ibc_core::msg::ExecuteMsg::IbcConfig { msg:cosmwasm_std::IbcChannelConnectMsg::OpenConfirm { channel: channel } }, &[])
}

pub fn call_register_connection(ctx: &mut TestContext)->Result<AppResponse, AppError> {
    

    ctx.app.execute_contract(ctx.sender.clone(), ctx.get_ibc_core(), 
    &cw_mock_ibc_core::msg::ExecuteMsg::RegisterXcall { address:ctx.get_xcall_ibc_connection() },&[])
}

// not possible without handshake
#[ignore]
#[test]
fn test_xcall_send_call_message() {
    let mut ctx = setup_test();
    call_set_xcall_host(&mut ctx).unwrap();
    call_register_connection(&mut ctx).unwrap();
    let src = ctx.get_xcall_ibc_connection().to_string();
    
    let nid="0x3.icon";
    call_configure_connection(&mut ctx, "connection-1".to_string(), nid.to_string(), "client-1".to_string()).unwrap();
    call_channel_connect(&mut ctx).unwrap();
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
    assert_eq!(&format!("{nid}/{MOCK_CONTRACT_TO_ADDR}"), event.get("to").unwrap());
}





