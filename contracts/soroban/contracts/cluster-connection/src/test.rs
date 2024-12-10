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
    bytesn, symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events}, token, vec, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec
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

    let val1 = bytesn!(&ctx.env, 0x04deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209);
    let val2 = bytesn!(&ctx.env, 0x04f9379b2955d759a9532f8daa0c4a25da0ae706dd057de02af7754adb4b956ec9b8bf7a8a5a6686bc74dff736442a874c6bae5dcbcdb7113e24fbfa2337c63a01);
    let val3 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);
    let val4 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(val1);
    validators.push_back(val2);
    validators.push_back(val3);
    validators.push_back(val4);
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

    let val1 = bytesn!(&ctx.env, 0x04deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209);
    let val2 = bytesn!(&ctx.env, 0x04f9379b2955d759a9532f8daa0c4a25da0ae706dd057de02af7754adb4b956ec9b8bf7a8a5a6686bc74dff736442a874c6bae5dcbcdb7113e24fbfa2337c63a01);
    let val3 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);
    let val4 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(val1);
    validators.push_back(val2);
    validators.push_back(val3);
    validators.push_back(val4);
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

    let val1 = bytesn!(&ctx.env, 0x04deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209);
    let val2 = bytesn!(&ctx.env, 0x04f9379b2955d759a9532f8daa0c4a25da0ae706dd057de02af7754adb4b956ec9b8bf7a8a5a6686bc74dff736442a874c6bae5dcbcdb7113e24fbfa2337c63a01);
    let val3 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);
    let val4 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(val1);
    validators.push_back(val2);
    validators.push_back(val3);
    validators.push_back(val4);
    client.update_validators(&validators, &2_u32);

    let conn_sn = 128_u128;
    let msg = Bytes::from_array(&ctx.env,&[104, 101, 108, 108, 111]);
    let src_network = String::from_str(&ctx.env, "0x2.icon");

    let mut signatures = Vec::new(&ctx.env);
    signatures.push_back(bytesn!(&ctx.env, 0x660d542b3f6de9cd08f238fd44133eeebfea290b21dae7322a63b516c57b8df12c4c0a340b60ed567c8da53578346c212b27b797eb42a75fb4b7076c567a6ff91c));
    signatures.push_back(bytesn!(&ctx.env, 0x8024de4c7b003df96bb699cfaa1bfb8a682787cd0853f555d48494c65c766f8104804848095890a9a6d15946da52dafb18e5c1d0dbe7f33fc7a5fa5cf8b1f6e21c));

    client.recv_message_with_signatures(&src_network, &conn_sn, &msg, &signatures);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_receive_message_less_signatures() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = bytesn!(&ctx.env, 0x04deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209);
    let val2 = bytesn!(&ctx.env, 0x04f9379b2955d759a9532f8daa0c4a25da0ae706dd057de02af7754adb4b956ec9b8bf7a8a5a6686bc74dff736442a874c6bae5dcbcdb7113e24fbfa2337c63a01);
    let val3 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);
    let val4 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(val1);
    validators.push_back(val2);
    validators.push_back(val3);
    validators.push_back(val4);
    client.update_validators(&validators, &2_u32);

    let conn_sn = 128_u128;
    let msg = Bytes::from_array(&ctx.env,&[104, 101, 108, 108, 111]);
    let src_network = String::from_str(&ctx.env, "0x2.icon");

    let mut signatures = Vec::new(&ctx.env);
    signatures.push_back(bytesn!(&ctx.env, 0x660d542b3f6de9cd08f238fd44133eeebfea290b21dae7322a63b516c57b8df12c4c0a340b60ed567c8da53578346c212b27b797eb42a75fb4b7076c567a6ff91c));

    client.recv_message_with_signatures(&src_network, &conn_sn, &msg, &signatures);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_receive_message_with_invalid_signature() {
    let ctx = TestContext::default();
    let client = ClusterConnectionClient::new(&ctx.env, &ctx.contract);

    ctx.init_context(&client);

    let val1 = bytesn!(&ctx.env, 0x04deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209);
    let val2 = bytesn!(&ctx.env, 0x04f9379b2955d759a9532f8daa0c4a25da0ae706dd057de02af7754adb4b956ec9b8bf7a8a5a6686bc74dff736442a874c6bae5dcbcdb7113e24fbfa2337c63a01);
    let val3 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);
    let val4 = bytesn!(&ctx.env, 0x041d7fa5b41fe40ae85130c4cc334f7852c25c19e7f326a916d49f6b9c3f35a1216bf53c805d177c28f7bedc2d2521cb0f13dc832ef689797965274d26df50cd0f);

    let mut validators = Vec::new(&ctx.env);
    validators.push_back(val1);
    validators.push_back(val2);
    validators.push_back(val3);
    validators.push_back(val4);
    client.update_validators(&validators, &2_u32);

    let conn_sn = 456456_u128;
    let msg = Bytes::from_array(&ctx.env,&[104, 100, 108, 108, 111]);
    let src_network = String::from_str(&ctx.env, "0x2.icon");

    let mut signatures = Vec::new(&ctx.env);
    signatures.push_back(bytesn!(&ctx.env, 0x660d542b3f6de9cd08f238fd44133eeebfea290b21dae7322a63b516c57b8df12c4c0a340b60ed567c8da53578346c212b27b797eb42a75fb4b7076c567a6ff91c));
    signatures.push_back(bytesn!(&ctx.env, 0x660d542b3f6de9cd08f238fd44133eeebfea290b21dae7322a63b516c57b8df12c4c0a340b60ed567c8da53578346c212b27b797eb42a75fb4b7076c567a6ff91c));

    client.recv_message_with_signatures(&src_network, &conn_sn, &msg, &signatures);
}

