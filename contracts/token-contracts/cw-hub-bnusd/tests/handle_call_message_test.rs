mod setup;
use std::str::FromStr;

use cw_common::{hub_token_msg::ExecuteMsg, x_call_msg::XCallMsg as XCallExecuteMsg};
use cw_multi_test::Executor;

use cosmwasm_std::{Addr, Uint128};
use cw_common::{data_types::CrossTransfer, network_address::NetworkAddress};

use crate::setup::{call_set_xcall_host, execute_setup, instantiate_contracts, mint_token};
use cw20::BalanceResponse;
use cw20_base::msg::QueryMsg;
use cw_common::network_address::NetId;
use cw_ibc_rlp_lib::rlp::{encode, RlpStream};
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

fn balance_of(context: &TestContext, user: Addr) -> BalanceResponse {
    context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
}

#[test]
pub fn cross_transfer_revert_data_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_setup(context);
    let x_call_connection = context.get_xcall_connection();
    context = set_default_connection(context, x_call_connection.clone());
    call_set_xcall_host(&mut context);

    let initial_balance = Uint128::from(u128::MIN + 1000);

    let sender = context.sender.to_string();
    context = mint_token(context, sender, initial_balance);
    let to = NetworkAddress::from_str("icon/cx9876543210fedcba9876543210fedcba98765432").unwrap();
    let amount = Uint128::from(u128::MIN + 100);
    let user = context.sender.clone();
    let response = context
        .app
        .execute_contract(
            user.clone(),
            context.get_hubtoken_app(),
            &ExecuteMsg::CrossTransfer {
                to,
                amount: amount.into(),
                data: vec![],
            },
            &[],
        )
        .unwrap();

    let event = get_event(&response, "wasm-CallMessageSent").unwrap();
    let sequence_no = event.get("sn").unwrap();

    let balance = balance_of(&context, user.clone());
    let expected_balance = (initial_balance - amount);
    assert_eq!(balance.balance, expected_balance);

    let message_type: u64 = 2;
    let mut stream = RlpStream::new();
    stream.begin_list(2);
    stream.append(&sequence_no.parse::<u64>().unwrap());
    stream.append(&0);

    let encoded_data: Vec<u8> = stream.out().to_vec();

    let mut stream = RlpStream::new();
    stream.begin_list(2);
    stream.append(&message_type);
    stream.append(&encoded_data);

    let data = stream.out().to_vec();

    context
        .app
        .execute_contract(
            x_call_connection.clone(),
            context.get_xcall_app(),
            &XCallExecuteMsg::HandleMessage {
                from: NetId::from("icon".to_owned()),
                msg: data,
            },
            &[],
        )
        .unwrap();

    context
        .app
        .execute_contract(
            user.clone(),
            context.get_xcall_app(),
            &XCallExecuteMsg::ExecuteRollback {
                sequence_no: sequence_no.parse::<u128>().unwrap(),
            },
            &[],
        )
        .unwrap();

    let balance = balance_of(&context, user.clone());
    println!("{:?}", balance.balance);
    assert_eq!(balance.balance, initial_balance);
}
