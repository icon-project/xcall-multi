mod setup;
use std::str::FromStr;

use cosmwasm_std::Uint128;
use cw_common::hub_token_msg::ExecuteMsg;
use cw_common::network_address::NetworkAddress;
use cw_multi_test::Executor;

use crate::setup::{call_set_xcall_host, execute_setup, instantiate_contracts};
use setup::{mint_token, set_default_connection, setup_context, TestContext};

pub fn cross_transfer(mut ctx: TestContext) -> TestContext {
    let x_call_connection = ctx.get_xcall_connection();
    ctx = set_default_connection(ctx, x_call_connection);
    call_set_xcall_host(&mut ctx);
    let _resp = ctx
        .app
        .execute_contract(
            ctx.sender.clone(),
            ctx.get_hubtoken_app(),
            &ExecuteMsg::CrossTransfer {
                to: NetworkAddress::from_str("icon/cx9876543210fedcba9876543210fedcba98765432")
                    .unwrap(),
                amount: 100,
                data: vec![],
            },
            &[],
        )
        .unwrap();
    println!("{:?}", _resp);

    ctx
}

#[test]
pub fn cross_transfer_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_setup(context);
    let sender = context.sender.to_string();
    context = mint_token(context, sender, Uint128::from(u128::MIN + 1000));
    cross_transfer(context);
}
