pub mod setup;
use std::str::FromStr;
use cosmwasm_std::Coin;
use cosmwasm_std::{testing::mock_env, Env};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_info, MockApi, MockQuerier},
    Addr, MemoryStorage, OwnedDeps, Uint128,
};
use cw_centralized_connection::{
    execute, msg::ExecuteMsg, state::CwCentralizedConnection, types::InstantiateMsg,
};
use cw_xcall_lib::network_address::NetId;

const XCALL: &str = "xcall";
const DENOM: &str = "denom";
const RELAYER: &str = "relayer";
const OWNER: &str = "owner";

fn instantiate(
    sender: &str,
) -> (
    OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
    Env,
    CwCentralizedConnection<'_>,
) {
    let mut deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier> = mock_dependencies();
    let mut ctx: CwCentralizedConnection<'_> = CwCentralizedConnection::default();
    let env = mock_env();
    let info = mock_info(sender, &[]);
    let msg = InstantiateMsg {
        relayer: RELAYER.to_string(),
        xcall_address: XCALL.to_string(),
        denom: DENOM.to_string(),
    };
    let res = ctx.instantiate(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(res.is_ok());

    (deps, env, ctx)
}

#[test]
fn test_initialization() {
    instantiate(OWNER);
}

#[test]
fn test_set_admin() {
    let (mut deps, env, _ctx) = instantiate("sender");
    let msg = ExecuteMsg::SetAdmin {
        address: Addr::unchecked("admin"),
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(!res.is_ok());

    let info = mock_info(RELAYER, &[]);

    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());
}

#[test]
fn test_set_fee() {
    let (mut deps, env, ctx) = instantiate(OWNER);
    let nid = NetId::from_str("test").unwrap();
    let message_fee: u128 = 200;
    let response_fee: u128 = 100;
    let msg = ExecuteMsg::SetFee {
        network_id: nid.clone(),
        message_fee,
        response_fee,
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(!res.is_ok());

    let info = mock_info(RELAYER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok());

    let res = ctx
        .get_fee(deps.as_mut().storage, nid.clone(), false)
        .unwrap();
    assert_eq!(res, Uint128::from(message_fee));

    let res = ctx.get_fee(deps.as_mut().storage, nid, true).unwrap();
    assert_eq!(res, Uint128::from(message_fee + response_fee));
}

#[test]
pub fn test_send_message() {
    let (mut deps, env, _ctx) = instantiate(OWNER);
    let msg = ExecuteMsg::SendMessage {
        to: NetId::from_str("nid").unwrap(),
        svc: "xcall".to_string(),
        sn: 0,
        msg: vec![],
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(!res.is_ok());

    let info = mock_info(XCALL, &[]);

    let res = execute(deps.as_mut(), env, info, msg);

    assert!(res.is_ok());
}

#[test]

pub fn test_recv_message() {
    let (mut deps, env, mut _ctx) = instantiate(OWNER);
    let src_network = NetId::from_str("nid").unwrap();
    let msg = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn: 1,
        msg: "".to_string(),
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(!res.is_ok());
    assert_eq!("Only Relayer(Admin)", res.unwrap_err().to_string());

    let info = mock_info(RELAYER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());

    assert!(res.is_ok());

    let res = execute(deps.as_mut(), env, info, msg);

    assert!(!res.is_ok());

    assert_eq!("Duplicate Message", res.unwrap_err().to_string());
}

#[test]

pub fn test_revert_message() {
    let (mut deps, env, mut _ctx) = instantiate(OWNER);
    let msg = ExecuteMsg::RevertMessage { sn: 1 };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(!res.is_ok());

    let info = mock_info(RELAYER, &[]);

    let res = execute(deps.as_mut(), env, info, msg);

    assert!(res.is_ok());
}

#[test]

pub fn test_get_receipts() {
    let (mut deps, env, ctx) = instantiate(OWNER);
    let src_network = NetId::from_str("nid").unwrap();
    let msg = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn: 1,
        msg: "".to_string(),
    };

    let receipt = ctx.get_receipt(deps.as_mut().storage, src_network.clone(), 1);
    assert!(!receipt);

    let _ = execute(deps.as_mut(), env, mock_info(RELAYER, &[]), msg);

    let receipt = ctx.get_receipt(deps.as_mut().storage, src_network, 1);
    assert!(receipt);
}

#[test]
pub fn test_claim_fees() {
    let (mut deps, env, _ctx) = instantiate(OWNER);
    let claim_msg = ExecuteMsg::ClaimFees {};
    let info = mock_info(OWNER, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, claim_msg.clone());
    assert!(!res.is_ok());
    assert_eq!("Only Relayer(Admin)", res.unwrap_err().to_string());

    let msg = ExecuteMsg::SendMessage {
        to: NetId::from_str("nid").unwrap(),
        svc: "xcall".to_string(),
        sn: 0,
        msg: vec![],
    };

    let info = mock_info(XCALL, &[]);

    let _ = execute(deps.as_mut(), env.clone(), info, msg.clone());

    let amount: u128 = 100;
    let coin: Coin = Coin { denom: DENOM.to_string(), amount: Uint128::from(amount)};
    let info = mock_info(RELAYER, &[coin]);
    let res = execute(deps.as_mut(), env, info, claim_msg);
    assert!(res.is_ok());
}
