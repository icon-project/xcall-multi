mod setup;
use cosmwasm_std::{Addr, Uint128};
use cw_common::{asset_manager_msg::ExecuteMsg, x_call_msg::XCallMsg as XCallExecuteMsg};
use cw_ibc_rlp_lib::rlp::RlpStream;
use cw_multi_test::Executor;
use cw_xcall_lib::network_address::NetId;

use crate::setup::{
    execute_config_x_call, get_event, instantiate_contracts, set_default_connection, setup_context,
    TestContext,
};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

//test helper
fn deposit_cw20_token(mut ctx: TestContext, msg: ExecuteMsg) -> TestContext {
    let xcall_connection = ctx.get_xcall_connection();
    ctx = set_default_connection(ctx, xcall_connection);

    let resp = ctx
        .app
        .execute_contract(ctx.sender.clone(), ctx.get_asset_manager_app(), &msg, &[]);

    println!("deposit execution resp: {:?}", resp);
    ctx
}

fn increase_allowance(mut ctx: TestContext, amount: Uint128) -> (TestContext, Uint128) {
    let xcall_connection = ctx.get_xcall_connection();
    let am_addr = ctx.get_asset_manager_app();

    let spoke_addr = ctx.get_cw20token_app();
    let token = Cw20Contract(ctx.get_cw20token_app());

    ctx = set_default_connection(ctx, xcall_connection);

    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: am_addr.to_string(),
        amount,
        expires: Some(cw_utils::Expiration::Never {}),
    };
    ctx.app
        .execute_contract(ctx.sender.clone(), spoke_addr, &allowance_msg, &[])
        .unwrap();
    let resp = token
        .allowance(&ctx.app.wrap(), ctx.sender.clone(), am_addr)
        .unwrap();

    (ctx, resp.allowance)
}

//check for manual test modification in only transfer sub msg atomic execution inside contract
fn check_balance(ctx: &TestContext, token: &Addr, account: &Addr) -> Uint128 {
    let token_contract = Cw20Contract(token.clone());
    let app_query_wrapper = ctx.app.wrap();
    token_contract.balance(&app_query_wrapper, account).unwrap()
}

#[test]
/**
Testing Contract's Addresses
* asset_manager -----> contract3
* spoke_token -----> contract1
* source_x_call -----> contract0
*/
fn test_deposit_expected_for_revert() {
    let mut context = setup_context();
    context = instantiate_contracts(context);
    let spoke_addr = context.get_cw20token_app();
    let source_x_call = context.get_xcall_app();

    context = execute_config_x_call(context, source_x_call);

    let deposit_msg = ExecuteMsg::Deposit {
        token_address: spoke_addr.to_string(),
        amount: Uint128::new(100),
        to: None,
        data: None,
    };

    let (ctx, allowance) = increase_allowance(context, Uint128::new(1000));
    assert_eq!(allowance, Uint128::new(1000));
    let ctx = deposit_cw20_token(ctx, deposit_msg);
    //balance will be updated after transfer on manual sub msg execution check
    let bl = check_balance(&ctx, &spoke_addr, &ctx.get_asset_manager_app());
    assert_eq!(Uint128::new(100), bl);
}

#[test]
fn test_deposit_revert() {
    let mut context = setup_context();
    context = instantiate_contracts(context);
    let spoke_addr = context.get_cw20token_app();
    let source_x_call = context.get_xcall_app();
    context = execute_config_x_call(context, source_x_call);
    let xcall_connection = context.get_xcall_connection();
    context = set_default_connection(context, xcall_connection.clone());

    let (mut ctx, _allowance) = increase_allowance(context, Uint128::new(1000));
    let user = ctx.sender.clone();

    let initial_balance = check_balance(&ctx, &spoke_addr, &user);

    let deposit_msg = ExecuteMsg::Deposit {
        token_address: spoke_addr.to_string(),
        amount: Uint128::new(100),
        to: None,
        data: None,
    };

    let response = ctx
        .app
        .execute_contract(
            ctx.sender.clone(),
            ctx.get_asset_manager_app(),
            &deposit_msg,
            &[],
        )
        .unwrap();

    let event = get_event(&response, "wasm-CallMessageSent").unwrap();
    let sequence_no = event.get("sn").unwrap();

    let bl = check_balance(&ctx, &spoke_addr, &user);
    assert_eq!(initial_balance - Uint128::new(100), bl);

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

    ctx.app
        .execute_contract(
            xcall_connection.clone(),
            ctx.get_xcall_app(),
            &XCallExecuteMsg::HandleMessage {
                from: NetId::from("icon".to_owned()),
                msg: data,
            },
            &[],
        )
        .unwrap();

    ctx.app
        .execute_contract(
            user.clone(),
            ctx.get_xcall_app(),
            &XCallExecuteMsg::ExecuteRollback {
                sequence_no: sequence_no.parse::<u128>().unwrap(),
            },
            &[],
        )
        .unwrap();

    let bl = check_balance(&ctx, &spoke_addr, &user);
    assert_eq!(initial_balance, bl);
}
