use soroban_rlp::encoder;
use soroban_sdk::{bytes, testutils::Address as _, vec, Address, String};
use soroban_xcall_lib::network_address::NetworkAddress;

use super::setup::*;
extern crate std;

use crate::{contract::MockDappClient, storage, types::Connection};

#[test]
fn test_init() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    ctx.env.as_contract(&ctx.contract, || {
        let addr = storage::get_xcall_address(&ctx.env).unwrap();
        assert_eq!(addr, ctx.xcall);
    });

    let sn = client.get_sequence();
    assert_eq!(sn, u128::default())
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_get_sequence_fail() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);

    client.get_sequence();
}

#[test]
fn test_send_call_message() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg_type = 0;
    let sender = Address::generate(&ctx.env);
    ctx.mint_native_token(&sender, 500);
    assert_eq!(ctx.get_native_token_balance(&sender), 500);

    let res = client.send_call_message(
        &ctx.network_address,
        &bytes!(&ctx.env, 0x00),
        &msg_type,
        &None,
        &sender,
    );
    assert_eq!(res, 1);
    assert_eq!(ctx.get_native_token_balance(&sender), 300)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_rollback_message_fail_for_empty_rollback_data() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg_type = 1;
    let sender = Address::generate(&ctx.env);
    client.send_call_message(
        &ctx.network_address,
        &bytes!(&ctx.env, 0xabc),
        &msg_type,
        &None,
        &sender,
    );
}

#[test]
fn test_send_rollback_message() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg_type = 1;
    let sender = Address::generate(&ctx.env);
    ctx.mint_native_token(&sender, 500);
    assert_eq!(ctx.get_native_token_balance(&sender), 500);

    client.send_call_message(
        &ctx.network_address,
        &bytes!(&ctx.env, 0xabc),
        &msg_type,
        &Some(bytes!(&ctx.env, 0xabc)),
        &sender,
    );
    assert_eq!(ctx.get_native_token_balance(&sender), 200)
}

#[test]
fn test_send_call_message_persisted() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let msg_type = 2;
    let sender = Address::generate(&ctx.env);
    ctx.mint_native_token(&sender, 500);
    assert_eq!(ctx.get_native_token_balance(&sender), 500);

    client.send_call_message(
        &ctx.network_address,
        &bytes!(&ctx.env, 0xabc),
        &msg_type,
        &None,
        &sender,
    );
    assert_eq!(ctx.get_native_token_balance(&sender), 300)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_call_message_fail_connections_not_found() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let nid = String::from_str(&ctx.env, "archway");
    let account = Address::generate(&ctx.env).to_string();
    let to = NetworkAddress::new(&ctx.env, nid, account);
    client.send_call_message(
        &to,
        &bytes!(&ctx.env, 0xabc),
        &1,
        &None,
        &Address::generate(&ctx.env),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_call_message_fail_xcall_address_not_set() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);

    ctx.env.mock_all_auths();

    let network_address = &ctx.network_address;
    let src = Address::generate(&ctx.env).to_string();
    let dst = Address::generate(&ctx.env).to_string();
    client.add_connection(&src, &dst, &network_address.nid(&ctx.env));

    ctx.env.as_contract(&ctx.contract, || {
        let connections =
            storage::get_connections(&ctx.env, network_address.nid(&ctx.env)).unwrap();
        assert_eq!(connections, vec![&ctx.env, Connection::new(src, dst)]);
    });

    client.send_call_message(
        network_address,
        &bytes!(&ctx.env, 0xabc),
        &1,
        &None,
        &Address::generate(&ctx.env),
    );
}

#[test]
fn test_handle_call_message_pass() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths();

    let from = NetworkAddress::new(&ctx.env, ctx.nid, ctx.xcall.to_string());
    let res = client.try_handle_call_message(
        &from.to_string(),
        &bytes!(&ctx.env, 0xabc),
        &Some(vec![&ctx.env]),
    );
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Ok(()))
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_handle_call_message_revert() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths();

    let string_data = String::from_str(&ctx.env, "rollback");
    let encoded_data = encoder::encode_string(&ctx.env, string_data);

    client.handle_call_message(
        &ctx.network_address.to_string(),
        &encoded_data,
        &Some(vec![&ctx.env]),
    );
}

#[test]
fn test_handle_call_message_reply() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);
    ctx.env.mock_all_auths_allowing_non_root_auth();

    ctx.mint_native_token(&ctx.contract, 500);

    let string_data = String::from_str(&ctx.env, "reply-response");
    let encoded_data = encoder::encode_string(&ctx.env, string_data);

    client.handle_call_message(
        &ctx.network_address.to_string(),
        &encoded_data,
        &Some(vec![&ctx.env]),
    );
}
