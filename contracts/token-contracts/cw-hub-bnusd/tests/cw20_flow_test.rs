use cosmwasm_std::{Addr, Uint128};
use cw20::{
    AllAccountsResponse, AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceResponse,
    BalanceResponse, MarketingInfoResponse, MinterResponse,
};
use cw20_base::state::TokenInfo;
use cw_multi_test::Executor;
use setup::{execute_setup, instantiate_contracts, setup_context, TestContext};

mod setup;

#[test]
fn cw20_flow_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_setup(context);

    let alice = Addr::unchecked("alice".to_owned());
    let bob = Addr::unchecked("bob".to_owned());
    let carol = Addr::unchecked("carol".to_owned());
    let amount: u128 = 1000;

    //mint 1000 tokens to each account, and minting access is given to only xcall app
    let resp = context.app.execute_contract(
        context.get_hubtoken_app(),
        context.get_xcall_app(),
        &cw_common::hub_token_msg::ExecuteMsg::Mint {
            recipient: alice.to_string(),
            amount: Uint128::from(amount),
        },
        &[],
    );

    assert!(resp.is_err()); //cannot mint tokens from hubtoken app

    //use loop to mint tokens from xcall app to alice, bob and carol
    vec![alice.to_string(), bob.to_string(), carol.to_string()]
        .iter()
        .for_each(|recipient| {
            let _resp = context
                .app
                .execute_contract(
                    context.get_xcall_app(),
                    context.get_hubtoken_app(),
                    &cw_common::hub_token_msg::ExecuteMsg::Mint {
                        recipient: recipient.clone(),
                        amount: Uint128::from(amount),
                    },
                    &[],
                )
                .unwrap();
        });

    //check balance of each account, and assert this to be 1000
    vec![
        (alice.to_string(), amount),
        (bob.to_string(), amount),
        (carol.to_string(), amount),
    ]
    .iter()
    .for_each(|(account, balance)| {
        let balance_response: BalanceResponse = context
            .app
            .wrap()
            .query_wasm_smart(
                context.get_hubtoken_app(),
                &cw_common::hub_token_msg::QueryMsg::Balance {
                    address: account.to_string(),
                },
            )
            .unwrap();
        println!("balance: {:?}", balance_response.balance.u128());
        assert_eq!(balance_response.balance.u128(), *balance);
    });

    let mut balances = [amount, amount, amount];

    //transfer 100 tokens from alice to bob and check again balance
    let transfer_amount: u128 = 100;
    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Transfer {
                recipient: bob.to_string(),
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[0] - transfer_amount
    );

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[1] + transfer_amount
    );

    balances = [
        balances[0] - transfer_amount,
        balances[1] + transfer_amount,
        balances[2],
    ];

    //transfer 100 tokens from bob to carol and check again balance
    let _resp = context
        .app
        .execute_contract(
            bob.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Transfer {
                recipient: carol.to_string(),
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[1] - transfer_amount
    );

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: carol.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[2] + transfer_amount
    );

    balances = [
        balances[0],
        balances[1] - transfer_amount,
        balances[2] + transfer_amount,
    ];

    //check self transfer, and the after the transfer amount should not increase
    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Transfer {
                recipient: alice.to_string(),
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_response.balance.u128(), balances[0]);

    let allowances_amount: u128 = 100;

    //set allowance of 100 tokens from alice to bob and and transfer 50 tokens of alice from bob to carol
    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::IncreaseAllowance {
                spender: bob.to_string(),
                amount: Uint128::from(allowances_amount),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_response.balance.u128(), balances[0]);

    let allowance_response: AllowanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Allowance {
                owner: alice.to_string(),
                spender: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(allowance_response.allowance.u128(), allowances_amount);

    let _resp = context
        .app
        .execute_contract(
            bob.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::TransferFrom {
                owner: alice.to_string(),
                recipient: carol.to_string(),
                amount: Uint128::from(transfer_amount / 2),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[0] - transfer_amount / 2
    );

    balances = [
        balances[0] - transfer_amount / 2,
        balances[1],
        balances[2] + transfer_amount / 2,
    ];

    //get allowance of alice to bob and assert it to be 50
    let allowance_response: AllowanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Allowance {
                owner: alice.to_string(),
                spender: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        allowance_response.allowance.u128(),
        allowances_amount - transfer_amount / 2
    );

    //increase, decrease and check allowance

    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::IncreaseAllowance {
                spender: bob.to_string(),
                amount: Uint128::from(transfer_amount),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let allowance_response: AllowanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Allowance {
                owner: alice.to_string(),
                spender: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        allowance_response.allowance.u128(),
        transfer_amount + transfer_amount / 2
    );

    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::DecreaseAllowance {
                spender: bob.to_string(),
                amount: Uint128::from(transfer_amount),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let allowance_response: AllowanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Allowance {
                owner: alice.to_string(),
                spender: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(allowance_response.allowance.u128(), transfer_amount / 2);

    //burn 100 tokens from alice and check balance
    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Burn {
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[0] - transfer_amount
    );

    balances = [balances[0] - transfer_amount, balances[1], balances[2]];

    println!("balances {:?}", balances);
    //burn_from test and check balance

    let _resp = context
        .app
        .execute_contract(
            bob.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::BurnFrom {
                owner: alice.to_string(),
                amount: Uint128::from(transfer_amount / 2),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance_response.balance.u128(),
        balances[0] - transfer_amount / 2
    );

    balances = [balances[0] - transfer_amount / 2, balances[1], balances[2]];

    //check allowance of bob to be 0
    let allowance_response: AllowanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Allowance {
                owner: alice.to_string(),
                spender: bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(allowance_response.allowance.u128(), 0);

    //update minter and check xcall app cannot mint tokens

    let _resp = context
        .app
        .execute_contract(
            context.get_xcall_app(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::UpdateMinter {
                new_minter: Some(bob.to_string()),
            },
            &[],
        )
        .unwrap();

    let resp: MinterResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Minter {},
        )
        .unwrap();
    println!("resp {:?}", resp);
    assert_eq!(resp.minter, bob.to_string());

    //try to mint by xcall which should fail

    let resp = context.app.execute_contract(
        context.get_xcall_app(),
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::ExecuteMsg::Mint {
            recipient: alice.to_string(),
            amount: Uint128::from(amount),
        },
        &[],
    );
    assert!(resp.is_err());

    //try to mint by bob which should pass and check balance of alice
    let _resp = context
        .app
        .execute_contract(
            bob.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Mint {
                recipient: alice.to_string(),
                amount: Uint128::from(amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::Balance {
                address: alice.to_string(),
            },
        )
        .unwrap();

    assert_eq!(balance_response.balance.u128(), balances[0] + amount);
    balances = [balances[0] + amount, balances[1], balances[2]];
    //all query tests

    let token_info: TokenInfo = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::TokenInfo {},
        )
        .unwrap();

    println!("token_info {:?}", token_info);
    let expected_info = TokenInfo {
        name: "Balanced Dollar".to_owned(),
        symbol: "bnUSD".to_owned(),
        decimals: 18,
        total_supply: Uint128::from(balances[0] + balances[1] + balances[2]),
        mint: None,
    };

    assert_eq!(token_info, expected_info);

    //query all allowances and all spender allowances

    let all_allowances_response: AllAllowancesResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::AllAllowances {
                owner: alice.to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    println!("all_allowances_response {:?}", all_allowances_response);

    //all spenderallowancesz

    let all_spender_allowances_response: AllSpenderAllowancesResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::AllSpenderAllowances {
                spender: bob.to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    println!(
        "all_spender_allowances_response {:?}",
        all_spender_allowances_response
    );

    //query all accounts

    let all_accounts: AllAccountsResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::AllAccounts {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    println!("all_accounts {:?}", all_accounts);

    //marketing info and download logo
    let marketing_info: MarketingInfoResponse = context
        .app
        .wrap()
        .query_wasm_smart(
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::QueryMsg::MarketingInfo {},
        )
        .unwrap();
    println!("marketing_info {:?}", marketing_info);
}
