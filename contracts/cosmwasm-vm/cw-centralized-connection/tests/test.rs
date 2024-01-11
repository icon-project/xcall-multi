pub mod setup;
use std::str::FromStr;
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
    let message_fee:u128 = 200;
    let response_fee:u128 = 100;
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

    let res = ctx.get_fee(deps.as_mut().storage, nid.clone(), false).unwrap();
    assert_eq!(res, Uint128::from(message_fee) );

    let res = ctx.get_fee(deps.as_mut().storage, nid, true).unwrap();
    assert_eq!(res, Uint128::from(message_fee+response_fee) );
}

#[test]
pub fn test_send_message(){
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

pub fn test_recv_message(){
    let (mut deps, env,mut _ctx) = instantiate(OWNER);
    let src_network = NetId::from_str("nid").unwrap();
    let msg = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn: 1,
        msg: vec![],
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

pub fn test_revert_message(){
    let (mut deps, env,mut _ctx) = instantiate(OWNER);
    let msg = ExecuteMsg::RevertMessage {
        sn: 1,
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(!res.is_ok());

    let info = mock_info(RELAYER, &[]);

    let res = execute(deps.as_mut(), env, info, msg);

    assert!(res.is_ok());
}

#[test]

pub fn test_get_receipts(){
    let (mut deps, env,ctx) = instantiate(OWNER);
    let src_network = NetId::from_str("nid").unwrap();
    let msg = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn: 1,
        msg: vec![],
    };

    let receipt = ctx.get_receipt(deps.as_mut().storage, src_network.clone(), 1);
    assert!(!receipt);

    let _ = execute(deps.as_mut(), env, mock_info(RELAYER, &[]), msg);

    let receipt = ctx.get_receipt(deps.as_mut().storage, src_network, 1);
    assert!(receipt);
}

#[test]
pub fn test_claim_fees(){
    let (mut deps, env, _ctx) = instantiate(OWNER);
    let msg = ExecuteMsg::ClaimFees {};
    let info = mock_info(OWNER, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(!res.is_ok());

    let info = mock_info(RELAYER, &[]);
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

}





// #[test]
// fn test_sequence() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();

//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     ctx.increment_sequence(&mut deps.storage).unwrap();
//     let res = ctx.get_sequence(&deps.storage);

//     assert!(res.is_ok());
//     assert_eq!(res.unwrap(), 1)
// }

// #[test]
// fn test_send_message() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();
//     let info = create_mock_info("hugobyte", "umlg", 2000);
//     let env = mock_env();

//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     let msg = InstantiateMsg {
//         address: "xcall-address".to_string(),
//     };
//     ctx.instantiate(deps.as_mut(), env, info.clone(), msg)
//         .unwrap();

//     let res = ctx.send_call_message(
//         deps.as_mut(),
//         info,
//         NetworkAddress::from_str("netid/xcall").unwrap(),
//         vec![1, 2, 3, 4],
//         Some(vec![1, 2, 3, 4, 5]),
//     );

//     assert!(res.is_ok());
//     assert_eq!(res.unwrap().messages[0].id, 0)
// }

// #[test]
// #[should_panic(expected = "ModuleAddressNotFound")]
// fn test_send_message_fail() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();
//     let info = create_mock_info("hugobyte", "umlg", 2000);
//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     ctx.send_call_message(
//         deps.as_mut(),
//         info,
//         NetworkAddress::from_str("netid/xcall").unwrap(),
//         vec![1, 2, 3, 4],
//         Some(vec![1, 2, 3, 4, 5]),
//     )
//     .unwrap();
// }

// #[test]
// fn test_handle_message() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();
//     let info = create_mock_info("hugobyte", "umlg", 2000);
//     let env = mock_env();

//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     let msg = InstantiateMsg {
//         address: "xcall-address".to_string(),
//     };
//     ctx.instantiate(deps.as_mut(), env, info.clone(), msg)
//         .unwrap();
//     let res = ctx.handle_call_message(
//         deps.as_mut(),
//         info,
//         NetworkAddress::from_str("netid/xcall").unwrap(),
//         "helloError".as_bytes().to_vec(),
//     );
//     assert!(res.is_ok())
// }

// #[test]
// #[should_panic(expected = "RevertFromDAPP")]
// fn test_handle_message_fail_revert() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();
//     let info = create_mock_info("hugobyte", "umlg", 2000);
//     let env = mock_env();

//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     let msg = InstantiateMsg {
//         address: "xcall-address".to_string(),
//     };
//     ctx.instantiate(deps.as_mut(), env, info.clone(), msg)
//         .unwrap();
//     ctx.handle_call_message(
//         deps.as_mut(),
//         info,
//         NetworkAddress::from_str("netid/xcall").unwrap(),
//         "rollback".as_bytes().to_vec(),
//     )
//     .unwrap();
// }

// #[test]
// fn test_handle_message_pass_true() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();
//     let info = create_mock_info("hugobyte", "umlg", 2000);
//     let env = mock_env();

//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     let msg = InstantiateMsg {
//         address: "xcall-address".to_string(),
//     };
//     ctx.instantiate(deps.as_mut(), env, info.clone(), msg)
//         .unwrap();

//     ctx.roll_back()
//         .save(&mut deps.storage, 1, &vec![1, 2, 3])
//         .unwrap();

//     let rollback_data = RollbackData {
//         id: 1,
//         rollback: vec![1, 2, 3],
//     };
//     let res = ctx.handle_call_message(
//         deps.as_mut(),
//         info,
//         NetworkAddress::from_str("netid/hugobyte").unwrap(),
//         to_vec(&rollback_data).unwrap(),
//     );
//     assert!(res.is_ok());
//     println!("{:?}", res);
//     assert_eq!(res.unwrap().attributes[0].value, "RollbackDataReceived")
// }

// #[test]
// #[should_panic(expected = "MisiingRollBack")]
// fn test_handle_message_fail_true() {
//     let mut deps = deps();
//     let ctx = CwCentralizedConnection::default();
//     let info = create_mock_info("hugobyte", "umlg", 2000);
//     let env = mock_env();

//     ctx.init_sequence(&mut deps.storage, u64::default())
//         .unwrap();
//     let msg = InstantiateMsg {
//         address: "xcall-address".to_string(),
//     };
//     ctx.instantiate(deps.as_mut(), env, info.clone(), msg)
//         .unwrap();

//     let rollback_data = RollbackData {
//         id: 1,
//         rollback: vec![1, 2, 3],
//     };
//     ctx.handle_call_message(
//         deps.as_mut(),
//         info,
//         NetworkAddress::from_str("netid/hugobyte").unwrap(),
//         to_vec(&rollback_data).unwrap(),
//     )
//     .unwrap();
// }
