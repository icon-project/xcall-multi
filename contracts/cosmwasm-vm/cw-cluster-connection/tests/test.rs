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
const ADMIN: &str = "admin";
const RELAYER: &str = "relayer";

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
        relayer: RELAYER.to_string(),
        xcall_address: XCALL.to_string(),
        denom: DENOM.to_string(),
    };
    let res = ctx.instantiate(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok());

    (deps, env, ctx)
}

#[test]
fn test_initialization() {
    instantiate(ADMIN);
}

#[test]
fn test_set_admin() {
    let (mut deps, env, ctx) = instantiate(ADMIN);
    let msg_info = mock_info(ADMIN, &[]);

    let new_admin = Addr::unchecked("new_admin");
    let msg = ExecuteMsg::SetAdmin {
        address: new_admin.clone(),
    };

    let res = execute(deps.as_mut(), env, msg_info, msg);
    assert!(res.is_ok());

    let admin = ctx.get_admin(deps.as_mut().storage).unwrap();
    assert_eq!(admin, new_admin);
}

#[test]
fn test_set_admin_unauthorized() {
    let (mut deps, env, ctx) = instantiate(ADMIN);
    let msg_info = mock_info("UnathorizedUser", &[]);

    let new_admin = Addr::unchecked("new_admin");
    let msg = ExecuteMsg::SetAdmin {
        address: new_admin.clone(),
    };

    let res = execute(deps.as_mut(), env, msg_info, msg);
    assert!(res.is_err());
    assert_eq!("Only Admin", res.unwrap_err().to_string());
}

#[test]
fn test_set_relayer() {
    let (mut deps, env, ctx) = instantiate(ADMIN);
    let msg_info = mock_info(ADMIN, &[]);

    let new_relayer = Addr::unchecked("new_relayer");
    let msg = ExecuteMsg::SetRelayer {
        address: new_relayer.clone(),
    };

    let res = execute(deps.as_mut(), env, msg_info, msg);
    assert!(res.is_ok());

    let relayer = ctx.get_relayer(deps.as_mut().storage).unwrap();
    assert_eq!(relayer, new_relayer);
}

#[test]
fn test_set_relayer_unauthorized() {
    let (mut deps, env, ctx) = instantiate(ADMIN);
    let msg_info = mock_info("UnathorizedUser", &[]);

    let new_relayer = Addr::unchecked("new_relayer");
    let msg = ExecuteMsg::SetRelayer {
        address: new_relayer.clone(),
    };

    let res = execute(deps.as_mut(), env, msg_info, msg);
    assert!(res.is_err());
    assert_eq!("Only Admin", res.unwrap_err().to_string());
}

#[test]
pub fn test_set_validators() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let validators = vec![
        "validator1".to_string(),
        "validator2".to_string(),
        "validator3".to_string(),
    ];

    let threshold = 2;

    let msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold: threshold,
    };

    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_ok());

    let stored_validators = ctx.get_validators(deps.as_ref().storage).unwrap();

    assert_eq!(stored_validators, validators);

    let stored_threshold = ctx.get_signature_threshold(deps.as_ref().storage);
    assert_eq!(stored_threshold, threshold);
}

#[test]
pub fn test_set_validators_unauthorized() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let validators = vec![
        "validator1".to_string(),
        "validator2".to_string(),
        "validator3".to_string(),
    ];

    let threshold = 2;

    let msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold: threshold,
    };

    let info = mock_info("UnauthorisedUser", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_err());
    assert_eq!("Only Admin", res.unwrap_err().to_string());
}

#[test]
pub fn test_set_validators_empty() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let validators = vec!["val1".to_string(), "val2".to_string(), "val3".to_string()];

    let info = mock_info(ADMIN, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::SetValidators {
            validators: validators.clone(),
            threshold: 2,
        },
    );
    assert!(res.is_ok());

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::SetValidators {
            validators: vec![],
            threshold: 2,
        },
    );
    assert!(res.is_ok());

    let stored_validators = ctx.get_validators(deps.as_ref().storage).unwrap();
    let stored_threshold = ctx.get_signature_threshold(deps.as_ref().storage);

    assert_eq!(stored_validators, validators);
    assert_eq!(stored_threshold, 2);
}

#[test]
pub fn test_set_validators_invalid_threshold() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let validators = vec!["val1".to_string(), "val2".to_string(), "val3".to_string()];

    let info = mock_info(ADMIN, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::SetValidators {
            validators: validators.clone(),
            threshold: 0,
        },
    );
    assert!(res.is_err());

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::SetValidators {
            validators: vec![],
            threshold: 4,
        },
    );
    assert!(res.is_err());
}

#[test]
fn test_set_fee() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let nid = NetId::from_str("0x2.icon").unwrap();
    let message_fee: u128 = 200;
    let response_fee: u128 = 100;

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        ExecuteMsg::SetFee {
            network_id: nid.clone(),
            message_fee,
            response_fee,
        },
    );
    assert!(res.is_ok());

    let res = ctx
        .get_fee(deps.as_mut().storage, nid.clone(), false)
        .unwrap();
    assert_eq!(res, Uint128::from(message_fee));

    let res = ctx.get_fee(deps.as_mut().storage, nid, true).unwrap();
    assert_eq!(res, Uint128::from(message_fee + response_fee));
}

#[test]
fn test_set_fee_unauthorized() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let nid = NetId::from_str("0x2.icon").unwrap();
    let message_fee: u128 = 200;
    let response_fee: u128 = 100;

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("UnauthorisedUser", &[]),
        ExecuteMsg::SetFee {
            network_id: nid.clone(),
            message_fee,
            response_fee,
        },
    );
    assert!(res.is_err());
    assert_eq!("Only Relayer", res.unwrap_err().to_string());
}

#[test]
pub fn test_claim_fees() {
    let (mut deps, env, _ctx) = instantiate(ADMIN);

    let nid = NetId::from_str("0x2.icon").unwrap();
    let message_fee: u128 = 200;
    let response_fee: u128 = 100;

    let amount: u128 = 200;
    let coin: Coin = Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(amount),
    };

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        ExecuteMsg::SetFee {
            network_id: nid.clone(),
            message_fee,
            response_fee,
        },
    );
    assert!(res.is_ok());

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(XCALL, &[coin.clone()]),
        ExecuteMsg::SendMessage {
            to: nid,
            sn: 0,
            msg: vec![],
        },
    );
    assert!(res.is_ok());

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        ExecuteMsg::ClaimFees {},
    );
    assert!(res.is_ok());
}

#[test]
pub fn test_send_message() {
    let (mut deps, env, _ctx) = instantiate(ADMIN);
    let msg = ExecuteMsg::SendMessage {
        to: NetId::from_str("nid").unwrap(),
        sn: 0,
        msg: vec![],
    };

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(XCALL, &[]),
        msg.clone(),
    );

    assert!(res.is_ok());
    let event = Event::new("Message")
        .add_attribute("targetNetwork", "nid")
        .add_attribute("connSn", 1.to_string())
        .add_attribute("msg", "null");
    assert_eq!(res.unwrap().events[0], event);
}

#[test]
pub fn test_send_message_unauthorized() {
    let (mut deps, env, _ctx) = instantiate(ADMIN);
    let msg = ExecuteMsg::SendMessage {
        to: NetId::from_str("nid").unwrap(),
        sn: 0,
        msg: vec![],
    };

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        msg.clone(),
    );

    assert!(res.is_err());

    assert_eq!("Only XCall", res.unwrap_err().to_string());
}

#[test]
pub fn test_recv_message() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let validators =
        vec!["02e5e9769497fbc7c7ee57ab39ccedcb612018577d30ca090033dc67ba5d68b8ab".to_string()];
    let set_validators_msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold: 1,
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
    let mut sign_1 = hex::decode("62249c41d09297800f35174e041ad53ec85c5dcad6a6bd0db3267d36a56eb92d7645b7a64c22ae7e1f93c6c3867d2a33e6534e64093600861916e3299e4cc922").unwrap();
    sign_1.push(1);
    let signatures = vec![sign_1];

    // Test with non-relayer sender (should fail)
    let msg_with_signatures = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn,
        msg: msg.to_string(),
        signatures: signatures.clone(),
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("WHO AM I", &[]),
        msg_with_signatures.clone(),
    );
    assert!(res.is_err());
    assert_eq!("Only Relayer", res.unwrap_err().to_string());

    // Test with relayer sender (should succeed)
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        msg_with_signatures.clone(),
    );
    assert!(res.is_ok());

    // Verify that the message was received (check receipt)
    let receipt = ctx.get_receipt(deps.as_ref().storage, src_network.clone(), conn_sn);
    assert!(receipt);
}

#[test]
pub fn test_recv_message_signatures_insufficient() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let validators = vec![
        "val1".to_string(),
        "02e5e9769497fbc7c7ee57ab39ccedcb612018577d30ca090033dc67ba5d68b8ab".to_string(),
    ];
    let set_validators_msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold: 2,
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
    let mut sign_1 = hex::decode("62249c41d09297800f35174e041ad53ec85c5dcad6a6bd0db3267d36a56eb92d7645b7a64c22ae7e1f93c6c3867d2a33e6534e64093600861916e3299e4cc922").unwrap();
    sign_1.push(1);
    let signatures = vec![sign_1];

    let msg_with_signatures = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn,
        msg: msg.to_string(),
        signatures: signatures.clone(),
    };

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        msg_with_signatures.clone(),
    );
    assert!(res.is_err());
    assert_eq!("Insufficient Signatures", res.unwrap_err().to_string());
}

fn string_to_hex(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
