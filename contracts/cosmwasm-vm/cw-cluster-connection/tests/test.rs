pub mod setup;
use cluster_connection::{
    execute, msg::ExecuteMsg, state::ClusterConnection, types::InstantiateMsg,
};
use cosmwasm_std::{testing::mock_env, Env};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_info, MockApi, MockQuerier},
    Addr, MemoryStorage, OwnedDeps, Uint128,
};
use cosmwasm_std::{Coin, Event};
use cw_xcall_lib::network_address::NetId;
use std::str::FromStr;

const XCALL: &str = "xcall";
const DENOM: &str = "denom";
const ADMIN: &str = "admin_relayer";
const OWNER: &str = "owner";

fn instantiate(
    sender: &str,
) -> (
    OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
    Env,
    ClusterConnection<'_>,
) {
    let mut deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier> = mock_dependencies();
    let mut ctx: ClusterConnection<'_> = ClusterConnection::default();
    let env = mock_env();
    let info = mock_info(sender, &[]);
    let msg = InstantiateMsg {
        admin: ADMIN.to_string(),
        xcall_address: XCALL.to_string(),
        denom: DENOM.to_string(),
    };
    let res = ctx.instantiate(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok());

    (deps, env, ctx)
}

#[test]
fn test_initialization() {
    instantiate(OWNER);
}

#[test]
fn test_set_admin() {
    let (mut deps, env, ctx) = instantiate("sender");
    let msg = ExecuteMsg::SetAdmin {
        address: Addr::unchecked("admin"),
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_err());

    let info = mock_info(ADMIN, &[]);

    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let admin = ctx.get_admin(deps.as_mut().storage).unwrap();
    assert_eq!(admin, Addr::unchecked("admin"));

    let validators = ctx.get_validators(deps.as_mut().storage).unwrap();
    assert_eq!(validators[0], Addr::unchecked("admin"));
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
    assert!(res.is_err());

    let info = mock_info(ADMIN, &[]);

    let res = execute(deps.as_mut(), env, info, msg);
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
        sn: 0,
        msg: vec![],
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(res.is_err());

    let info: cosmwasm_std::MessageInfo = mock_info(XCALL, &[]);

    let res = execute(deps.as_mut(), env, info, msg);
    let event = Event::new("Message")
        .add_attribute("targetNetwork", "nid")
        .add_attribute("connSn", 1.to_string())
        .add_attribute("msg", "null");
    assert_eq!(res.unwrap().events[0], event);
}

#[test]
pub fn test_recv_message() {
    let (mut deps, env, mut _ctx) = instantiate(OWNER);
    let src_network = NetId::from_str("nid").unwrap();
    let msg = ExecuteMsg::RecvMessage {
        src_network,
        conn_sn: 1,
        msg: "".to_string(),
    };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(res.is_err());
    assert_eq!("Only Relayer(Admin)", res.unwrap_err().to_string());

    let info = mock_info(ADMIN, &[]);

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());

    assert!(res.is_ok());

    let res = execute(deps.as_mut(), env, info, msg);

    assert!(res.is_err());

    assert_eq!("Duplicate Message", res.unwrap_err().to_string());
}

#[test]

pub fn test_revert_message() {
    let (mut deps, env, mut _ctx) = instantiate(OWNER);
    let msg = ExecuteMsg::RevertMessage { sn: 1 };

    let info = mock_info(OWNER, &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(res.is_err());

    let info = mock_info(ADMIN, &[]);

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

    let _ = execute(deps.as_mut(), env, mock_info(ADMIN, &[]), msg);

    let receipt = ctx.get_receipt(deps.as_mut().storage, src_network, 1);
    assert!(receipt);
}

#[test]
pub fn test_claim_fees() {
    let (mut deps, env, _ctx) = instantiate(OWNER);
    let claim_msg = ExecuteMsg::ClaimFees {};
    let info = mock_info(OWNER, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, claim_msg.clone());
    assert!(res.is_err());
    assert_eq!("Only Relayer(Admin)", res.unwrap_err().to_string());

    let msg = ExecuteMsg::SendMessage {
        to: NetId::from_str("nid").unwrap(),
        sn: 0,
        msg: vec![],
    };

    let info = mock_info(XCALL, &[]);

    let _ = execute(deps.as_mut(), env.clone(), info, msg);

    let amount: u128 = 100;
    let coin: Coin = Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(amount),
    };
    let info = mock_info(ADMIN, &[coin]);
    let res = execute(deps.as_mut(), env, info, claim_msg);
    assert!(res.is_ok());
}

#[test]
pub fn test_set_validators() {
    let (mut deps, env, ctx) = instantiate(OWNER);

    // Prepare validators
    let validators = vec![
        "validator1".to_string(),
        "validator2".to_string(),
        "validator3".to_string(),
    ];

    let msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
    };

    // Test with non-admin sender (should fail)
    let info = mock_info(OWNER, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_err());
    assert_eq!("Only Relayer(Admin)", res.unwrap_err().to_string());

    // Test with admin sender (should succeed)
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok());

    // Verify that validators were set correctly
    let stored_validators = ctx.get_validators(deps.as_ref().storage).unwrap();

    assert_eq!(stored_validators, validators);

    // Verify that we can't set an empty list of validators
    let msg = ExecuteMsg::SetValidators { validators: vec![] };
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok()); // The function allows setting an empty list, but the admin is always included

    let stored_validators = ctx.get_validators(deps.as_ref().storage).unwrap();
    assert_eq!(stored_validators, vec![Addr::unchecked(ADMIN)]);
}

#[test]
pub fn test_recv_message_with_signatures() {
    let (mut deps, env, ctx) = instantiate(OWNER);

    let validators =
        vec!["02e5e9769497fbc7c7ee57ab39ccedcb612018577d30ca090033dc67ba5d68b8ab".to_string()];
    let set_validators_msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
    };
    let _ = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[]),
        set_validators_msg,
    );

    // Set up test data
    let src_network = NetId::from_str("0x2.icon").unwrap();
    let conn_sn: u128 = 1;
    let msg = string_to_hex("hello");
    let sign_1 = hex::decode("62249c41d09297800f35174e041ad53ec85c5dcad6a6bd0db3267d36a56eb92d7645b7a64c22ae7e1f93c6c3867d2a33e6534e64093600861916e3299e4cc922").unwrap();
    let signatures = vec![sign_1];

    // Test with non-admin sender (should fail)
    let msg_with_signatures = ExecuteMsg::RecvMessageWithSignatures {
        src_network: src_network.clone(),
        conn_sn,
        msg: msg.to_string(),
        signatures: signatures.clone(),
    };
    let info = mock_info(OWNER, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        msg_with_signatures.clone(),
    );
    assert!(res.is_err());
    assert_eq!("Only Relayer(Admin)", res.unwrap_err().to_string());

    // Test with admin sender (should succeed)
    let info = mock_info(ADMIN, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        msg_with_signatures.clone(),
    );
    assert!(res.is_ok());

    // Verify that the message was received (check receipt)
    let receipt = ctx.get_receipt(deps.as_ref().storage, src_network.clone(), conn_sn);
    assert!(receipt);
}

fn string_to_hex(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
