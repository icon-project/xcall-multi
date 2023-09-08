mod setup;
use std::str::FromStr;

use cw_common::{data_types::CrossTransferRevert, x_call_msg::XCallMsg as XCallExecuteMsg};
use cw_multi_test::Executor;

use cosmwasm_std::{Addr, Uint128};
use cw_common::{data_types::CrossTransfer, network_address::NetworkAddress};

use crate::setup::{execute_setup, instantiate_contracts};
use cw20::BalanceResponse;
use cw20_base::msg::QueryMsg;
use cw_common::network_address::NetId;
use rlp::{encode, RlpStream};
use setup::{get_event, set_default_connection, setup_context, TestContext};

fn execute_and_handle_message(mut context: TestContext) -> TestContext {
    let hub_token_addr = context.get_hubtoken_app().into_string();
    let call_data = CrossTransfer {
        method: "xCrossTransfer".to_string(),
        from: NetworkAddress::from_str("icon/cx7866543210fedcba9876543210fedcba987654df").unwrap(),
        to: NetworkAddress::from_str("icon/cx9876543210fedcba9876543210fedcba98765432").unwrap(),
        value: 1000,
        data: vec![
            118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
        ],
    };

    let data = encode(&call_data).to_vec();

    let network_address =
        NetworkAddress::from_str("icon/cx7866543210fedcba9876543210fedcba987654df").unwrap();
    let sequence_no: u64 = 1234;
    let message_type: u64 = 1;

    let mut stream = RlpStream::new();
    stream.begin_list(6);
    stream.append(&network_address.to_string());
    stream.append(&hub_token_addr);
    stream.append(&sequence_no);
    stream.append(&false);
    stream.append(&data);
    stream.begin_list(0);

    let encoded_data: Vec<u8> = stream.out().to_vec();
    println!("Encoded Data {:?}", encoded_data);

    let mut stream = RlpStream::new();
    stream.begin_list(2);
    stream.append(&message_type);
    stream.append(&encoded_data);

    let data = stream.out().to_vec();

    let relay = Addr::unchecked("relay");
    context = set_default_connection(context, relay.clone());

    let response = context
        .app
        .execute_contract(
            relay,
            context.get_xcall_app(),
            &XCallExecuteMsg::HandleMessage {
                from: NetId::from("icon".to_owned()),
                msg: data,
            },
            &[],
        )
        .unwrap();

    let event = get_event(&response, "wasm-CallMessage").unwrap();
    let request_id = event.get("reqId").unwrap();
    println!("Request ID {:?}", request_id);

    let call_data = CrossTransfer {
        method: "xCrossTransfer".to_string(),
        from: NetworkAddress::from_str("icon/cx7866543210fedcba9876543210fedcba987654df").unwrap(),
        to: NetworkAddress::from_str("icon/cx9876543210fedcba9876543210fedcba98765432").unwrap(),
        value: 1000,
        data: vec![
            118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
        ],
    };

    let data = encode(&call_data).to_vec();

    let response = context
        .app
        .execute_contract(
            context.get_hubtoken_app(),
            context.get_xcall_app(),
            &XCallExecuteMsg::ExecuteCall {
                request_id: request_id.parse::<u128>().unwrap(),
                data,
            },
            &[],
        )
        .unwrap();

    let balance: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &QueryMsg::Balance {
                address: call_data.to.account().to_string(),
            },
        )
        .unwrap();
    let expected_balance = Uint128::from(u128::MIN + 1000);
    assert_eq!(balance.balance, expected_balance);

    println!("Response {:?}", response);

    context
}
/// HandleCallMessage is called by XCALL contract.
///
/// For testing purpose, we called HandleMessage of XCall from sender 'relay', which returns a request ID in
/// the response. This request ID is then used to ExecuteCall message, which in turn calls HandleCallMessage
/// of hubToken contract.

#[test]
fn handle_call_message_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_setup(context);
    execute_and_handle_message(context);
}

#[test]
pub fn cross_transfer_revert_data_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_setup(context);
    let hub_token_addr = context.get_hubtoken_app().into_string();

    let call_data = CrossTransferRevert {
        method: "xCrossTransferRevert".to_string(),
        from: Addr::unchecked("cx7866543210fedcba9876543210fedcba987654df".to_owned()),
        value: 1000,
    };

    let data = encode(&call_data).to_vec();

    let network_address =
        NetworkAddress::from_str("icon/cx7866543210fedcba9876543210fedcba987654df").unwrap();
    let sequence_no: u64 = 1234;
    let message_type: u64 = 1;

    let mut stream = RlpStream::new();
    stream.begin_list(6);
    stream.append(&network_address.to_string());
    stream.append(&hub_token_addr);
    stream.append(&sequence_no);
    stream.append(&false);
    stream.append(&data);
    stream.begin_list(0);

    let encoded_data: Vec<u8> = stream.out().to_vec();
    println!("Encoded Data {:?}", encoded_data);

    let mut stream = RlpStream::new();
    stream.begin_list(2);
    stream.append(&message_type);
    stream.append(&encoded_data);

    let data = stream.out().to_vec();

    let relay = Addr::unchecked("relay");
    context = set_default_connection(context, relay.clone());

    let response = context.app.execute_contract(
        relay,
        context.get_xcall_app(),
        &XCallExecuteMsg::HandleMessage {
            from: NetId::from("icon".to_owned()),
            msg: data,
        },
        &[],
    );

    println!("Response {:?}", response);
    let event = get_event(&response.unwrap(), "wasm-CallMessage").unwrap();
    let request_id = event.get("reqId").unwrap();
    println!("Request ID {:?}", request_id);

    let data = encode(&call_data).to_vec();

    let response = context
        .app
        .execute_contract(
            context.get_hubtoken_app(),
            context.get_xcall_app(),
            &XCallExecuteMsg::ExecuteCall {
                request_id: request_id.parse::<u128>().unwrap(),
                data,
            },
            &[],
        )
        .unwrap();

    let balance: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &QueryMsg::Balance {
                address: call_data.from.to_string(),
            },
        )
        .unwrap();
    let expected_balance = Uint128::from(u128::MIN + 1000);
    assert_eq!(balance.balance, expected_balance);

    println!("Response {:?}", response);
}
