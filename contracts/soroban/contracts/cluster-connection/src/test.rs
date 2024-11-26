#![cfg(test)]

extern crate std;

mod xcall {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/xcall.wasm");
}

use crate::{
    contract::{ClusterConnection, ClusterConnectionClient},
    event::SendMsgEvent,
    storage,
    types::InitializeMsg,
};
use soroban_sdk::{
    symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events}, token, vec, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec
};

pub struct TestContext {
    env: Env,
    xcall: Address,
    contract: Address,
    admin:Address,
    relayer: Address,
    native_token: Address,
    token_admin: Address,
    nid: String,
    upgrade_authority: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        let xcall = env.register_contract_wasm(None, xcall::WASM);
        Self {
            xcall: xcall.clone(),
            contract: env.register_contract(None, ClusterConnection),
            relayer: Address::generate(&env),
            admin: Address::generate(&env),
            native_token: env.register_stellar_asset_contract(token_admin.clone()),
            nid: String::from_str(&env, "0x2.icon"),
            upgrade_authority: Address::generate(&env),
            env,
            token_admin,
        }
    }

    pub fn init_context(&self, client: &ClusterConnectionClient<'static>) {
        self.env.mock_all_auths();

        client.initialize(&InitializeMsg {
            admin: self.admin.clone(),
            relayer: self.relayer.clone(),
            native_token: self.native_token.clone(),
            xcall_address: self.xcall.clone(),
            upgrade_authority: self.upgrade_authority.clone(),
        });

    }

    pub fn init_send_message(&self, client: &ClusterConnectionClient<'static>) {
        self.init_context(&client);
        self.env.mock_all_auths_allowing_non_root_auth();

        client.set_fee(&self.nid, &100, &100);
    }
}

fn get_dummy_initialize_msg(env: &Env) -> InitializeMsg {
    InitializeMsg {
        admin: Address::generate(&env),
        relayer: Address::generate(&env),
        native_token: env.register_stellar_asset_contract(Address::generate(&env)),
        xcall_address: Address::generate(&env),
        upgrade_authority: Address::generate(&env),
    }
}


#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let admin = client.get_admin();
    assert_eq!(admin, ctx.admin)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_initialize_fail_on_double_initialize() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    client.initialize(&get_dummy_initialize_msg(&ctx.env));
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let new_admin = Address::generate(&ctx.env);
    client.set_admin(&new_admin);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("set_admin"),
                    (new_admin.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    )
}

#[test]
#[should_panic]
fn test_set_admin_fail() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let new_admin = Address::generate(&ctx.env);
    client.set_admin(&new_admin);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.xcall,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("set_admin"),
                    (new_admin.clone(),).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    )
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_context(&client);

    let new_upgrade_authority = Address::generate(&ctx.env);
    client.set_upgrade_authority(&new_upgrade_authority);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.upgrade_authority.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.contract.clone(),
                    Symbol::new(&ctx.env, "set_upgrade_authority"),
                    (&new_upgrade_authority,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    let autorhity = client.get_upgrade_authority();
    assert_eq!(autorhity, new_upgrade_authority);
}

#[test]
fn test_set_fee() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let nid = String::from_str(&ctx.env, "icon");
    client.set_fee(&nid, &10, &10);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.relayer,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("set_fee"),
                    (nid.clone(), 10_u128, 10_u128).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_fee(&nid, &true), 20);
    assert_eq!(client.get_fee(&nid, &false), 10);
}

#[test]
fn test_claim_fees() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let token_client = token::Client::new(&ctx.env, &ctx.native_token);
    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);

    asset_client.mint(&ctx.contract, &1000);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.token_admin,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.native_token.clone(),
                    symbol_short!("mint"),
                    (&ctx.contract.clone(), 1000_i128,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token_client.balance(&ctx.contract), 1000);

    client.claim_fees();
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.relayer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "claim_fees"),
                    ().into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token_client.balance(&ctx.relayer), 1000);
    assert_eq!(token_client.balance(&ctx.contract), 0);
    assert_eq!(ctx.env.auths(), std::vec![]);
}

#[test]
fn test_send_message() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_send_message(&client);

    let tx_origin = Address::generate(&ctx.env);

    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);
    asset_client.mint(&tx_origin, &1000);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.send_message(&tx_origin, &ctx.nid, &1, &msg);

    assert_eq!(
        ctx.env.auths(),
        std::vec![
            (
                ctx.xcall.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        client.address.clone(),
                        Symbol::new(&ctx.env, "send_message"),
                        (tx_origin.clone(), ctx.nid.clone(), 1_i64, msg.clone()).into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                tx_origin.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.native_token.clone(),
                        Symbol::new(&ctx.env, "transfer"),
                        (tx_origin.clone(), ctx.contract.clone(), 200_i128).into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            )
        ]
    );

    let emit_msg = SendMsgEvent {
        targetNetwork: ctx.nid.clone(),
        connSn: 1_u128,
        msg: msg.clone(),
    };
    let event = vec![&ctx.env, ctx.env.events().all().last_unchecked()];
    assert_eq!(
        event,
        vec![
            &ctx.env,
            (
                client.address.clone(),
                ("Message",).into_val(&ctx.env),
                emit_msg.into_val(&ctx.env)
            )
        ]
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_send_message_fail_for_insufficient_fee() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);
    ctx.init_send_message(&client);

    let sender = Address::generate(&ctx.env);

    let asset_client = token::StellarAssetClient::new(&ctx.env, &ctx.native_token);
    asset_client.mint(&sender, &100);

    let msg = Bytes::from_array(&ctx.env, &[1, 2, 3]);
    client.send_message(&sender, &ctx.nid, &1, &msg);
}

#[test]
fn test_get_receipt_returns_false() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    let sequence_no = 1;
    let receipt = client.get_receipt(&ctx.nid, &sequence_no);
    assert_eq!(receipt, false);

    ctx.env.as_contract(&ctx.contract, || {
        storage::store_receipt(&ctx.env, ctx.nid.clone(), sequence_no);
    });

    let receipt = client.get_receipt(&ctx.nid, &sequence_no);
    assert_eq!(receipt, true)
}

#[test]
fn test_add_validator() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = [4, 174, 54, 168, 191, 216, 207, 101, 134, 243, 76, 104, 133, 40, 137, 72, 53, 245, 231, 193, 157, 54, 104, 155, 172, 84, 96, 101, 107, 97, 60, 94, 171, 241, 250, 152, 34, 18, 170, 39, 202, 236, 226, 58, 39, 8, 235, 60, 137, 54, 225, 50, 185, 253, 130, 197, 174, 226, 170, 75, 6, 145, 123, 87, 19];
    let val2 = [4, 91, 65, 155, 222, 192, 210, 187, 193, 108, 232, 174, 20, 79, 248, 232, 37, 18, 63, 208, 203, 62, 54, 208, 7, 91, 109, 141, 229, 170, 181, 51, 136, 172, 143, 180, 194, 138, 138, 56, 67, 243, 7, 60, 218, 164, 12, 148, 63, 116, 115, 127, 192, 206, 164, 169, 95, 135, 119, 138, 255, 172, 115, 129, 144];
    let val3 = [4, 248, 192, 175, 198, 228, 250, 20, 158, 23, 251, 176, 244, 208, 150, 71, 151, 27, 208, 22, 41, 30, 154, 198, 109, 10, 112, 142, 200, 47, 200, 213, 210, 172, 135, 141, 129, 183, 211, 241, 211, 127, 16, 19, 67, 159, 195, 235, 88, 164, 223, 47, 128, 47, 147, 28, 121, 28, 93, 129, 176, 144, 52, 243, 55];

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(BytesN::from_array(&ctx.env, &val1));
    validators.push_back(BytesN::from_array(&ctx.env, &val2));
    validators.push_back(BytesN::from_array(&ctx.env, &val3));
    client.update_validators(&validators, &3_u32);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "update_validators"),
                    (validators.clone(), 3_u32,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(
        client.get_validators(),
        validators
    );
}


#[test]
fn test_set_threshold() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = [4, 174, 54, 168, 191, 216, 207, 101, 134, 243, 76, 104, 133, 40, 137, 72, 53, 245, 231, 193, 157, 54, 104, 155, 172, 84, 96, 101, 107, 97, 60, 94, 171, 241, 250, 152, 34, 18, 170, 39, 202, 236, 226, 58, 39, 8, 235, 60, 137, 54, 225, 50, 185, 253, 130, 197, 174, 226, 170, 75, 6, 145, 123, 87, 19];
    let val2 = [4, 91, 65, 155, 222, 192, 210, 187, 193, 108, 232, 174, 20, 79, 248, 232, 37, 18, 63, 208, 203, 62, 54, 208, 7, 91, 109, 141, 229, 170, 181, 51, 136, 172, 143, 180, 194, 138, 138, 56, 67, 243, 7, 60, 218, 164, 12, 148, 63, 116, 115, 127, 192, 206, 164, 169, 95, 135, 119, 138, 255, 172, 115, 129, 144];
    let val3 = [4, 248, 192, 175, 198, 228, 250, 20, 158, 23, 251, 176, 244, 208, 150, 71, 151, 27, 208, 22, 41, 30, 154, 198, 109, 10, 112, 142, 200, 47, 200, 213, 210, 172, 135, 141, 129, 183, 211, 241, 211, 127, 16, 19, 67, 159, 195, 235, 88, 164, 223, 47, 128, 47, 147, 28, 121, 28, 93, 129, 176, 144, 52, 243, 55];

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(BytesN::from_array(&ctx.env, &val1));
    validators.push_back(BytesN::from_array(&ctx.env, &val2));
    validators.push_back(BytesN::from_array(&ctx.env, &val3));
    client.update_validators(&validators, &3_u32);

    let threshold: u32 = 2_u32;
    client.set_validators_threshold(&threshold);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&ctx.env, "set_validators_threshold"),
                    (threshold,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_validators_threshold(), threshold);

    let threshold: u32 = 3_u32;
    client.set_validators_threshold(&threshold);
    assert_eq!(client.get_validators_threshold(), threshold);
}


#[test]
fn test_receive_message() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = [4, 174, 54, 168, 191, 216, 207, 101, 134, 243, 76, 104, 133, 40, 137, 72, 53, 245, 231, 193, 157, 54, 104, 155, 172, 84, 96, 101, 107, 97, 60, 94, 171, 241, 250, 152, 34, 18, 170, 39, 202, 236, 226, 58, 39, 8, 235, 60, 137, 54, 225, 50, 185, 253, 130, 197, 174, 226, 170, 75, 6, 145, 123, 87, 19];
    let val2 = [4, 91, 65, 155, 222, 192, 210, 187, 193, 108, 232, 174, 20, 79, 248, 232, 37, 18, 63, 208, 203, 62, 54, 208, 7, 91, 109, 141, 229, 170, 181, 51, 136, 172, 143, 180, 194, 138, 138, 56, 67, 243, 7, 60, 218, 164, 12, 148, 63, 116, 115, 127, 192, 206, 164, 169, 95, 135, 119, 138, 255, 172, 115, 129, 144];
    let val3 = [4, 248, 192, 175, 198, 228, 250, 20, 158, 23, 251, 176, 244, 208, 150, 71, 151, 27, 208, 22, 41, 30, 154, 198, 109, 10, 112, 142, 200, 47, 200, 213, 210, 172, 135, 141, 129, 183, 211, 241, 211, 127, 16, 19, 67, 159, 195, 235, 88, 164, 223, 47, 128, 47, 147, 28, 121, 28, 93, 129, 176, 144, 52, 243, 55];

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(BytesN::from_array(&ctx.env, &val1));
    validators.push_back(BytesN::from_array(&ctx.env, &val2));
    validators.push_back(BytesN::from_array(&ctx.env, &val3));
    client.update_validators(&validators, &1_u32);

    let conn_sn = 456456_u128;
    let msg = Bytes::from_array(&ctx.env,&[104, 101, 108, 108, 111]);
    let src_network = String::from_str(&ctx.env, "0x2.icon");

    let mut signatures = Vec::new(&ctx.env);
    signatures.push_back(BytesN::from_array(&ctx.env, &[35, 247, 49, 199, 251, 53, 83, 51, 115, 148, 35, 48, 85, 203, 185, 236, 5, 171, 221, 29, 247, 203, 190, 195, 208, 218, 204, 237, 88, 191, 91, 75, 48, 87, 108, 161, 75, 234, 147, 234, 65, 134, 233, 32, 249, 159, 43, 159, 86, 211, 1, 117, 176, 167, 53, 99, 34, 243, 165, 215, 93, 232, 67, 184, 27]));

    client.recv_message_with_signatures(&src_network, &conn_sn, &msg, &signatures);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_receive_message_less_signatures() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = [4, 174, 54, 168, 191, 216, 207, 101, 134, 243, 76, 104, 133, 40, 137, 72, 53, 245, 231, 193, 157, 54, 104, 155, 172, 84, 96, 101, 107, 97, 60, 94, 171, 241, 250, 152, 34, 18, 170, 39, 202, 236, 226, 58, 39, 8, 235, 60, 137, 54, 225, 50, 185, 253, 130, 197, 174, 226, 170, 75, 6, 145, 123, 87, 19];
    let val2 = [4, 91, 65, 155, 222, 192, 210, 187, 193, 108, 232, 174, 20, 79, 248, 232, 37, 18, 63, 208, 203, 62, 54, 208, 7, 91, 109, 141, 229, 170, 181, 51, 136, 172, 143, 180, 194, 138, 138, 56, 67, 243, 7, 60, 218, 164, 12, 148, 63, 116, 115, 127, 192, 206, 164, 169, 95, 135, 119, 138, 255, 172, 115, 129, 144];
    let val3 = [4, 248, 192, 175, 198, 228, 250, 20, 158, 23, 251, 176, 244, 208, 150, 71, 151, 27, 208, 22, 41, 30, 154, 198, 109, 10, 112, 142, 200, 47, 200, 213, 210, 172, 135, 141, 129, 183, 211, 241, 211, 127, 16, 19, 67, 159, 195, 235, 88, 164, 223, 47, 128, 47, 147, 28, 121, 28, 93, 129, 176, 144, 52, 243, 55];

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(BytesN::from_array(&ctx.env, &val1));
    validators.push_back(BytesN::from_array(&ctx.env, &val2));
    validators.push_back(BytesN::from_array(&ctx.env, &val3));
    client.update_validators(&validators, &2_u32);

    let conn_sn = 456456_u128;
    let msg = Bytes::from_array(&ctx.env,&[104, 101, 108, 108, 111]);
    let src_network = String::from_str(&ctx.env, "0x2.icon");

    let mut signatures = Vec::new(&ctx.env);
    signatures.push_back(BytesN::from_array(&ctx.env, &[35, 247, 49, 199, 251, 53, 83, 51, 115, 148, 35, 48, 85, 203, 185, 236, 5, 171, 221, 29, 247, 203, 190, 195, 208, 218, 204, 237, 88, 191, 91, 75, 48, 87, 108, 161, 75, 234, 147, 234, 65, 134, 233, 32, 249, 159, 43, 159, 86, 211, 1, 117, 176, 167, 53, 99, 34, 243, 165, 215, 93, 232, 67, 184, 27]));

    client.recv_message_with_signatures(&src_network, &conn_sn, &msg, &signatures);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_receive_message_with_invalid_signature() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = [4, 174, 54, 168, 191, 216, 207, 101, 134, 243, 76, 104, 133, 40, 137, 72, 53, 245, 231, 193, 157, 54, 104, 155, 172, 84, 96, 101, 107, 97, 60, 94, 171, 241, 250, 152, 34, 18, 170, 39, 202, 236, 226, 58, 39, 8, 235, 60, 137, 54, 225, 50, 185, 253, 130, 197, 174, 226, 170, 75, 6, 145, 123, 87, 19];
    let val2 = [4, 91, 65, 155, 222, 192, 210, 187, 193, 108, 232, 174, 20, 79, 248, 232, 37, 18, 63, 208, 203, 62, 54, 208, 7, 91, 109, 141, 229, 170, 181, 51, 136, 172, 143, 180, 194, 138, 138, 56, 67, 243, 7, 60, 218, 164, 12, 148, 63, 116, 115, 127, 192, 206, 164, 169, 95, 135, 119, 138, 255, 172, 115, 129, 144];
    let val3 = [4, 248, 192, 175, 198, 228, 250, 20, 158, 23, 251, 176, 244, 208, 150, 71, 151, 27, 208, 22, 41, 30, 154, 198, 109, 10, 112, 142, 200, 47, 200, 213, 210, 172, 135, 141, 129, 183, 211, 241, 211, 127, 16, 19, 67, 159, 195, 235, 88, 164, 223, 47, 128, 47, 147, 28, 121, 28, 93, 129, 176, 144, 52, 243, 55];

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(BytesN::from_array(&ctx.env, &val1));
    validators.push_back(BytesN::from_array(&ctx.env, &val2));
    validators.push_back(BytesN::from_array(&ctx.env, &val3));
    client.update_validators(&validators, &1_u32);

    let conn_sn = 456456_u128;
    let msg = Bytes::from_array(&ctx.env,&[104, 100, 108, 108, 111]);
    let src_network = String::from_str(&ctx.env, "0x2.icon");

    let mut signatures = Vec::new(&ctx.env);
    signatures.push_back(BytesN::from_array(&ctx.env, &[35, 247, 49, 199, 251, 53, 83, 51, 115, 148, 35, 48, 85, 203, 185, 236, 5, 171, 221, 29, 247, 203, 190, 195, 208, 218, 204, 237, 88, 191, 91, 75, 48, 87, 108, 161, 75, 234, 147, 234, 65, 134, 233, 32, 249, 159, 43, 159, 86, 211, 1, 117, 176, 167, 53, 99, 34, 243, 165, 215, 93, 232, 67, 184, 27]));

    client.recv_message_with_signatures(&src_network, &conn_sn, &msg, &signatures);
}

