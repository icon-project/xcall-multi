mod setup;
use cosmwasm_std::{Addr, Uint128};
use cw_common::asset_manager_msg::ExecuteMsg;
use cw_multi_test::Executor;

use crate::setup::{
    execute_config_x_call, instantiate_contracts, set_default_connection, setup_context,
    TestContext,
};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

//test helper
fn deposit_cw20_token(mut ctx: TestContext, msg: ExecuteMsg) -> TestContext {
    let relay = ctx.get_xcall_connection();
    ctx = set_default_connection(ctx, relay);

    let resp = ctx
        .app
        .execute_contract(ctx.sender.clone(), ctx.get_asset_manager_app(), &msg, &[]);

    println!("deposit execution resp: {:?}", resp);
    ctx
}

fn increase_allowance(mut ctx: TestContext, amount: Uint128) -> (TestContext, Uint128) {
    let relay = ctx.get_xcall_connection();
    let am_addr = ctx.get_asset_manager_app();

    let spoke_addr = ctx.get_cw20token_app();
    let token = Cw20Contract(ctx.get_cw20token_app());

    ctx = set_default_connection(ctx, relay);

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
