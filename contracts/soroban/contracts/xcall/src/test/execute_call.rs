#![cfg(test)]

use crate::{contract::XcallClient, storage, types::rollback::Rollback};
use soroban_sdk::{bytes, testutils::Address as _, Address};

use super::setup::*;

#[test]
#[should_panic(expected = "HostError: Error(Contract, #14)")]
fn test_execute_rollback_fail_for_invalid_sequence_number() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    let sequence_no = 1;
    client.execute_rollback(&sequence_no);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_execute_rollback_fail_not_enabled() {
    let ctx = TestContext::default();
    let client = XcallClient::new(&ctx.env, &ctx.contract);

    let sequence_no = 1;
    let rollback = Rollback::new(
        Address::generate(&ctx.env),
        ctx.network_address,
        get_dummy_sources(&ctx.env),
        bytes!(&ctx.env, 0xabc),
        false,
    );

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_rollback(&ctx.env, sequence_no, &rollback);
    });

    client.execute_rollback(&sequence_no);
}
