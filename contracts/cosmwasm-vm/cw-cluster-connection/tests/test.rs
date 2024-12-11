pub mod setup;
use cluster_connection::{
    execute, msg::ExecuteMsg, state::ClusterConnection, types::InstantiateMsg,
};
use cluster_connection::{keccak256, SignableMsg};
use cosmwasm_std::{testing::mock_env, ContractResult, Env};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_info, MockApi, MockQuerier},
    Addr, MemoryStorage, OwnedDeps, Uint128,
};
use cosmwasm_std::{to_json_binary, Coin, ContractInfoResponse, Event, SystemResult, WasmQuery};
use cw_xcall_lib::network_address::NetId;
use k256::{ecdsa::SigningKey, ecdsa::VerifyingKey, elliptic_curve::rand_core::OsRng};
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

    let val1 = "02e27e3817bf0b6d451004609c2a5d29fe315dc1d1017500399fab540785958b7a";
    let val2 = "03ea8d2913ce5bb5637fe732f920ccee7a454a8f1c32a531e7abc1a58a23cc8db0";
    let val3 = "03cc5598f8f40103592b6ed9e04adcf9bd67fe06d677bf5b392af0ad9b553a5b16";
    let validators = vec![
        hex::decode(val1).unwrap(),
        hex::decode(val2).unwrap(),
        hex::decode(val3).unwrap(),
    ];

    let threshold = 2;

    let msg = ExecuteMsg::SetValidators {
        validators,
        threshold,
    };

    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_ok());

    let mut stored_validators = ctx.get_validators(deps.as_ref().storage).unwrap();
    let mut set_validators = [val1.to_string(), val2.to_string(), val3.to_string()];

    assert_eq!(stored_validators.sort(), set_validators.sort());

    let stored_threshold = ctx.get_signature_threshold(deps.as_ref().storage);
    assert_eq!(stored_threshold, threshold);
}

#[test]
pub fn test_set_validators_unauthorized() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let val1 = "02e27e3817bf0b6d451004609c2a5d29fe315dc1d1017500399fab540785958b7a";
    let val2 = "03ea8d2913ce5bb5637fe732f920ccee7a454a8f1c32a531e7abc1a58a23cc8db0";
    let val3 = "03cc5598f8f40103592b6ed9e04adcf9bd67fe06d677bf5b392af0ad9b553a5b16";
    let validators = vec![
        hex::decode(val1).unwrap(),
        hex::decode(val2).unwrap(),
        hex::decode(val3).unwrap(),
    ];

    let threshold = 2;

    let msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold,
    };

    let info = mock_info("UnauthorisedUser", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_err());
    assert_eq!("Only Admin", res.unwrap_err().to_string());
}

#[test]
pub fn test_set_validators_empty() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let val1 = "02e27e3817bf0b6d451004609c2a5d29fe315dc1d1017500399fab540785958b7a";
    let val2 = "03ea8d2913ce5bb5637fe732f920ccee7a454a8f1c32a531e7abc1a58a23cc8db0";
    let val3 = "03cc5598f8f40103592b6ed9e04adcf9bd67fe06d677bf5b392af0ad9b553a5b16";
    let validators = vec![
        hex::decode(val1).unwrap(),
        hex::decode(val2).unwrap(),
        hex::decode(val3).unwrap(),
    ];

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

    let mut stored_validators = ctx.get_validators(deps.as_ref().storage).unwrap();
    let stored_threshold = ctx.get_signature_threshold(deps.as_ref().storage);

    let mut set_validators = [val1.to_string(), val2.to_string(), val3.to_string()];
    assert_eq!(set_validators.sort(), stored_validators.sort());

    assert_eq!(stored_threshold, 2);
}

#[test]
pub fn test_set_validators_invalid_threshold() {
    let (mut deps, env, ctx) = instantiate(ADMIN);

    let val1 = "02e27e3817bf0b6d451004609c2a5d29fe315dc1d1017500399fab540785958b7a";
    let val2 = "03ea8d2913ce5bb5637fe732f920ccee7a454a8f1c32a531e7abc1a58a23cc8db0";
    let val3 = "03cc5598f8f40103592b6ed9e04adcf9bd67fe06d677bf5b392af0ad9b553a5b16";
    let validators = vec![
        hex::decode(val1).unwrap(),
        hex::decode(val2).unwrap(),
        hex::decode(val3).unwrap(),
    ];

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

    deps.querier.update_wasm(|r: &WasmQuery| match r {
        WasmQuery::Smart {
            contract_addr: _,
            msg: _,
        } => SystemResult::Ok(ContractResult::Ok(to_json_binary("archway/xcall").unwrap())),
        WasmQuery::ContractInfo { contract_addr: _ } => SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&ContractInfoResponse::default()).unwrap(),
        )),
        _ => todo!(),
    });

    let src_network = NetId::from_str("0x2.icon").unwrap();
    let dst_network = NetId::from_str("archway").unwrap();
    let conn_sn: u128 = 456456;
    let msg = string_to_hex("hello");

    let signed_msg = SignableMsg {
        src_network: src_network.to_string(),
        conn_sn,
        data: hex::decode(msg.clone()).unwrap(),
        dst_network: dst_network.to_string(),
    };
    let signed_msg = signed_msg.encode_utf8_bytes().to_vec();
    let message_digest = keccak256(&signed_msg);

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    let pubkey = verifying_key.to_encoded_point(false).as_bytes().to_vec();
    let (signature, recovery_code) = signing_key
        .sign_digest_recoverable(message_digest.clone())
        .unwrap();
    let mut sign_0 = signature.to_vec();
    sign_0.push(recovery_code.to_byte());

    let signing_key_1 = SigningKey::random(&mut OsRng);
    let verifying_key_1 = VerifyingKey::from(&signing_key_1);
    let pubkey_1 = verifying_key_1.to_encoded_point(false).as_bytes().to_vec();
    let (signature_1, recovery_code_1) = signing_key_1
        .sign_digest_recoverable(message_digest.clone())
        .unwrap();
    let mut sign_1 = signature_1.to_vec();
    sign_1.push(recovery_code_1.to_byte());

    let validators = vec![pubkey.clone(), pubkey_1.clone()];

    let set_validators_msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold: 2,
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[]),
        set_validators_msg,
    );
    assert!(res.is_ok());

    let signatures = vec![sign_1, sign_0];

    // Test with non-relayer sender (should fail)
    let msg_with_signatures = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn,
        msg,
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

    deps.querier.update_wasm(|r: &WasmQuery| match r {
        WasmQuery::Smart {
            contract_addr: _,
            msg: _,
        } => SystemResult::Ok(ContractResult::Ok(to_json_binary("archway/xcall").unwrap())),
        WasmQuery::ContractInfo { contract_addr: _ } => SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&ContractInfoResponse::default()).unwrap(),
        )),
        _ => todo!(),
    });

    let src_network = NetId::from_str("0x2.icon").unwrap();
    let dst_network = NetId::from_str("archway").unwrap();
    let conn_sn: u128 = 456456;
    let msg = string_to_hex("hello");

    let signed_msg = SignableMsg {
        src_network: src_network.to_string(),
        conn_sn,
        data: hex::decode(msg.clone()).unwrap(),
        dst_network: dst_network.to_string(),
    };
    let signed_msg = signed_msg.encode_utf8_bytes().to_vec();
    let message_digest = keccak256(&signed_msg);

    let signing_key = SigningKey::random(&mut OsRng);

    let verifying_key = VerifyingKey::from(&signing_key);
    let pubkey = verifying_key.to_encoded_point(false).as_bytes().to_vec();

    let (signature, recovery_code) = signing_key.sign_digest_recoverable(message_digest).unwrap();

    let mut sign_1 = signature.to_vec();
    sign_1.push(recovery_code.to_byte());

    let val2 = "03ea8d2913ce5bb5637fe732f920ccee7a454a8f1c32a531e7abc1a58a23cc8db0";
    let validators = vec![pubkey, hex::decode(val2).unwrap()];

    let set_validators_msg = ExecuteMsg::SetValidators {
        validators: validators.clone(),
        threshold: 2,
    };
    let resp = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[]),
        set_validators_msg,
    );
    assert!(resp.is_ok());

    let signatures = vec![sign_1];

    let msg_with_signatures = ExecuteMsg::RecvMessage {
        src_network: src_network.clone(),
        conn_sn,
        msg,
        signatures: signatures.clone(),
    };

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(RELAYER, &[]),
        msg_with_signatures.clone(),
    );
    println!("response: {:?}", res);
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
