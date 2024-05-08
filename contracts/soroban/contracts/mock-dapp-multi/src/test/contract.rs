use soroban_rlp::encoder;
use soroban_sdk::{bytes, testutils::Address as _, vec, Address, String};

use super::setup::*;

use crate::{
    contract::{MockDapp, MockDappClient},
    types::Connection,
};

#[test]
fn test_init() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);

    let xcall_addr = Address::generate(&ctx.env);
    client.init(&xcall_addr);

    ctx.env.as_contract(&ctx.contract, || {
        let addr = MockDapp::get_xcall_address(&ctx.env).unwrap();
        assert_eq!(xcall_addr, addr);
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
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_call_message_fail_connections_not_found() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);

    client.send_call_message(
        &ctx.network_address,
        &bytes!(&ctx.env, 0xabc),
        &1,
        &None,
        &u128::default(),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_send_call_message_fail_xcall_address_not_set() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);

    let network_address = &ctx.network_address;
    let src = Address::generate(&ctx.env).to_string();
    let dst = Address::generate(&ctx.env).to_string();
    client.add_connection(&src, &dst, &network_address.nid(&ctx.env));

    ctx.env.as_contract(&ctx.contract, || {
        let connections =
            MockDapp::get_connections(&ctx.env, network_address.nid(&ctx.env)).unwrap();
        assert_eq!(connections, vec![&ctx.env, Connection::new(src, dst)]);
    });

    client.send_call_message(
        &network_address,
        &bytes!(&ctx.env, 0xabc),
        &1,
        &None,
        &u128::default(),
    );
}

#[test]
fn test_handle_call_message_pass() {
    let ctx = TestContext::default();
    let client = MockDappClient::new(&ctx.env, &ctx.contract);

    let from = ctx.network_address;
    let sender = &from.account(&ctx.env);
    let res = client.try_handle_call_message(
        &Address::from_string(&sender),
        &from,
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

    let string_data = String::from_str(&ctx.env, "rollback");
    let encoded_data = encoder::encode_string(&ctx.env, string_data);

    client.handle_call_message(
        &Address::generate(&ctx.env),
        &ctx.network_address,
        &encoded_data,
        &Some(vec![&ctx.env]),
    );
}

#[test]
fn test_handle_call_message_reply() {}
