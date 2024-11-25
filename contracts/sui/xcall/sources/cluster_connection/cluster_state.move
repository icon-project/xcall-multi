module xcall::cluster_state {
    use std::string::{String};
    use sui::vec_map::{Self, VecMap};
    use xcall::xcall_utils::{Self as utils};
    use sui::coin::{Self};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    use sui::bag::{Bag, Self};
    use sui::event;
    use sui::address::{Self};
    use sui::hash::{Self};
    use 0x2::ecdsa_k1::{secp256k1_ecrecover, decompress_pubkey};
    
    //ERRORS
    const VerifiedSignaturesLessThanThreshold: u64 = 100;
    const NotEnoughSignatures: u64 = 101;
    const InvalidThreshold: u64 = 102;
    const ValidatorCountMustBeGreaterThanThreshold: u64 = 105;
    const InvalidAdminCap: u64 = 106;

    /* hash algorithm*/
    const KECCAK256: u8 = 0x00;
    const SHA256: u8 = 0x01;

    //EVENTS
    public struct ValidatorSetAdded has copy, drop {
        validators: vector<vector<u8>>,
        threshold: u64
    }

    public struct AdminCap has key,store {
        id: UID,
        connection_id: String
    }

    public(package) fun create_admin_cap(connection_id:String,ctx: &mut TxContext):AdminCap {
         let admin = AdminCap {
            id: object::new(ctx),
            connection_id: connection_id
        };
        admin
    }

    public(package) fun validate_admin_cap(self:&AdminCap,connection_id:String){
        assert!(self.connection_id == connection_id, InvalidAdminCap);
    }

    public(package) fun get_state_mut(states:&mut Bag,connection_id:String):&mut State {
      let state:&mut State=bag::borrow_mut(states,connection_id);
      state
    }

    public fun get_state(states:&Bag,connection_id:String):&State {
      let state:&State=bag::borrow(states,connection_id);
      state
    }
    

    public struct ReceiptKey has copy, drop, store {
        conn_sn: u128,
        nid: String,
    }

    public struct State has store{ 
        message_fee: VecMap<String, u64>,
        response_fee: VecMap<String, u64>,
        receipts: VecMap<ReceiptKey, bool>,
        conn_sn: u128,
        balance: Balance<SUI>,
        validators: vector<vector<u8>>,
        validators_threshold:u64,

    }

    public(package) fun create(): State {
        State {
            message_fee: vec_map::empty<String, u64>(),
            response_fee: vec_map::empty<String, u64>(),
            conn_sn: 0,
            receipts: vec_map::empty(),
            balance:balance::zero(),
            validators: vector::empty(),
            validators_threshold:0
        }
    }

    public(package) fun get_next_conn_sn(self:&mut State):u128 {
        let sn=self.conn_sn+1;
        self.conn_sn=sn;
        sn
    }

    public fun get_fee(self: &State, netId: &String, need_response: bool): u64 {
        let fee: u64 = if (need_response == true) {
            utils::get_or_default(&self.message_fee, netId, 0)
                + utils::get_or_default(&self.response_fee, netId, 0)
        } else {
            utils::get_or_default(&self.message_fee, netId, 0)
        };
        fee
    }

    public(package) fun set_fee(self: &mut State, net_id: String, message_fee: u64, response_fee: u64,caller:address) {
        if (vec_map::contains(&self.message_fee,&net_id)){
            vec_map::remove(&mut self.message_fee,&net_id);
        };
        if (vec_map::contains(&self.response_fee,&net_id)){
            vec_map::remove(&mut self.response_fee,&net_id);
        };
        vec_map::insert(&mut self.message_fee, net_id, message_fee);
        vec_map::insert(&mut self.response_fee, net_id, response_fee);
    }

    public(package) fun check_save_receipt(self: &mut State, net_id: String, sn: u128) {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        assert!(!vec_map::contains(&self.receipts, &receipt_key), 100);
        vec_map::insert(&mut self.receipts, receipt_key, true);
    }

    public(package) fun get_receipt(self: &State, net_id: String, sn: u128): bool {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        vec_map::contains(&self.receipts, &receipt_key)
    }

    public(package) fun deposit(self:&mut State,balance:Balance<SUI>){
        balance::join(&mut self.balance,balance);

    }
    
    public(package) fun claim_fees(self:&mut State,ctx:&mut TxContext){
        let total= self.balance.withdraw_all();
        let coin= coin::from_balance(total,ctx);
        transfer::public_transfer(coin,ctx.sender());

    }

    public(package) fun verify_signatures(
    self: &State,
    src_net_id: String,
    sn: u128,
    msg: vector<u8>,
    signatures: vector<vector<u8>>
    ) {
        let message_hash = utils::get_message_hash(src_net_id, sn, msg);
        let threshold = self.get_validator_threshold();
        let validators = self.get_validators();

        // Ensure the number of signatures meets the threshold
        assert!(signatures.length() >= threshold, NotEnoughSignatures);

        let mut i = 0;
        let mut unique_verified_pubkey = vector::empty();

        while (i < signatures.length()) {
            let mut signature = *signatures.borrow(i);
            let mut recovery_code = signature.pop_back();
            let code = 27 as u8;

            if (recovery_code >= code) {
                recovery_code = recovery_code - code;
            };

            signature.push_back(recovery_code);

            let pub_key = decompress_pubkey(
                &secp256k1_ecrecover(&signature, &message_hash, KECCAK256)
            );

            if (validators.contains(&pub_key)) {
                if (!unique_verified_pubkey.contains(&pub_key)) {
                    unique_verified_pubkey.push_back(pub_key);
                };

                // Exit early if the threshold is met
                if (unique_verified_pubkey.length() >= threshold) {
                    return;
                };
            };

            i = i + 1;
        };

        // Assert that the unique verified public keys meet the threshold
        assert!(
            unique_verified_pubkey.length() >= threshold,
            VerifiedSignaturesLessThanThreshold
        );
    }


    public(package) fun get_validator_threshold(self:&State):u64{
        self.validators_threshold
    }

    public(package) fun set_validator_threshold(self:&mut State,threshold:u64){
        assert!(threshold <= self.validators.length(), InvalidThreshold);
        self.validators_threshold=threshold
    }

    public(package) fun set_validators(self:&mut State,validator_pub_keys:vector<vector<u8>>,threshold:u64){
        self.validators=vector::empty();
        let mut validator_pub_keys = validator_pub_keys;
        while (validator_pub_keys.length() > 0) {
            let validator = validator_pub_keys.pop_back();
            if(self.validators.contains(&validator)){
                continue
            };
            self.validators.push_back(validator);
        };
        assert!(self.validators.length() >= threshold, ValidatorCountMustBeGreaterThanThreshold);
        self.validators_threshold=threshold;
        event::emit(ValidatorSetAdded { validators: self.validators, threshold });
    }

    public(package) fun get_validators(self:&State):vector<vector<u8>>{
        self.validators
    }

    #[test_only]
    public(package) fun create_state():State{
        State {
            message_fee: vec_map::empty<String, u64>(),
            response_fee: vec_map::empty<String, u64>(),
            conn_sn: 0,
            receipts: vec_map::empty(),
            balance:balance::zero(),
            validators: vector::empty(),
            validators_threshold:0
        }
    }
   
}

#[test_only]
module xcall::cluster_state_tests {
    use xcall::cluster_state::{State, get_validators, get_validator_threshold, set_validator_threshold, set_validators, verify_signatures};
    use sui::test_scenario::{Self, Scenario};

    #[test]
    fun test_add_validator(): State {
        let mut state = xcall::cluster_state::create_state();

        let validators = vector[
            x"045b419bdec0d2bbc16ce8ae144ff8e825123fd0cb3e36d0075b6d8de5aab53388ac8fb4c28a8a3843f3073cdaa40c943f74737fc0cea4a95f87778affac738190",
            x"04ae36a8bfd8cf6586f34c688528894835f5e7c19d36689bac5460656b613c5eabf1fa982212aa27caece23a2708eb3c8936e132b9fd82c5aee2aa4b06917b5713",
            x"04f8c0afc6e4fa149e17fbb0f4d09647971bd016291e9ac66d0a708ec82fc8d5d2ac878d81b7d3f1d37f1013439fc3eb58a4df2f802f931c791c5d81b09034f337",
            x"046bc928ee4932efd619ec4c00e0591e932cf2cfef13a59f6027da1c6cba36b35d91238b54aece19825025a9c7cb0bc58a60d5c49e7fc8e5b39fcc4c2193f5feb2"
        ];

        set_validators(&mut state, validators, 2);

        let validators = get_validators(&state);
        assert!((validators.length() == 4));


        set_validator_threshold(&mut state, 3);
        assert!(get_validator_threshold(&state)==3);

        state
    }


    #[test]
    fun test_set_get_threshold(): State {
        let mut state = test_add_validator();
        set_validator_threshold(&mut state, 1);
        assert!(get_validator_threshold(&state)==1);
        state
    }

    #[test]
    #[expected_failure(abort_code = 102)]
    fun test_set_threshold_too_high(): State {
        let mut state = test_set_get_threshold();
        set_validator_threshold(&mut state, 5);
        state
    }

    #[test]
    fun test_get_fee(): State {
        let mut state = xcall::cluster_state::create_state();
        xcall::cluster_state::set_fee(&mut state, b"net1".to_string(), 100, 50, @0xadd);
        xcall::cluster_state::set_fee(&mut state, b"net2".to_string(), 200, 100, @0xadd);

        let fee_without_response = xcall::cluster_state::get_fee(&state, &b"net1".to_string(), false);
        assert!(fee_without_response == 100);

        let fee_with_response = xcall::cluster_state::get_fee(&state, &b"net1".to_string(), true);
        assert!(fee_with_response == 150);

        state
    }

    #[test]
    fun test_update_fee(): State {
        let mut state = xcall::cluster_state::create_state();
        xcall::cluster_state::set_fee(&mut state, b"net1".to_string(), 200, 100, @0xadd);

        let fee = xcall::cluster_state::get_fee(&state, &b"net1".to_string(), true);
        assert!(fee == 300); // 200 message_fee + 100 response_fee

        // Update the fee
        xcall::cluster_state::set_fee(&mut state, b"net1".to_string(), 300, 200, @0xadd);
        let updated_fee = xcall::cluster_state::get_fee(&state, &b"net1".to_string(), true);
        assert!(updated_fee == 500); // 300 message_fee + 200 response_fee

        state
    }

    #[test]
    fun test_receipts(): State {
        let mut state = xcall::cluster_state::create_state();
        let sn = xcall::cluster_state::get_next_conn_sn(&mut state);
        
        xcall::cluster_state::check_save_receipt(&mut state, b"net1".to_string(), sn);
        let receipt_exists = xcall::cluster_state::get_receipt(&state, b"net1".to_string(), sn);
        assert!(receipt_exists == true);

        state
    }

    #[test]
    #[expected_failure(abort_code = 101)]
    fun test_verify_signatures_less_than_threshold(): State {
        let state = test_add_validator();
        let msg: vector<u8> = x"68656c6c6f";
        let src_net_id = b"0x2.icon".to_string();
        let conn_sn = 456456;

        let signatures = vector[x"23f731c7fb3553337394233055cbb9ec05abdd1df7cbbec3d0dacced58bf5b4b30576ca14bea93ea4186e920f99f2b9f56d30175b0a7356322f3a5d75de843b81b",
                                ];

        xcall::cluster_state::verify_signatures(&state,src_net_id, conn_sn, msg, signatures);
        state
    }

    #[test]
    #[expected_failure(abort_code = 100)]
    fun test_verify_signatures_invalid(): State {
        let state = test_set_get_threshold();
        let msg: vector<u8> = x"68656c6c6f";
        let src_net_id = b"0x2.icon".to_string();
        let conn_sn = 456456;

        let signatures = vector[x"23f731c7fb3553337394233055cbb9ec05abdd1df7cbbec3d0dacced58bf5b4b30576ca14bea93ea4186e920f99f2b9f56d30175b0a7356322f3a5d75de843b81c",
                                ];

        xcall::cluster_state::verify_signatures(&state,src_net_id, conn_sn, msg, signatures);
        state
    }

    #[test]
    fun test_verify_signatures(): State {
        let state = test_set_get_threshold();
        let msg: vector<u8> = x"68656c6c6f";
        let src_net_id = b"0x2.icon".to_string();
        let conn_sn = 456456;

        let signatures = vector[x"23f731c7fb3553337394233055cbb9ec05abdd1df7cbbec3d0dacced58bf5b4b30576ca14bea93ea4186e920f99f2b9f56d30175b0a7356322f3a5d75de843b81b",
                                ];

        xcall::cluster_state::verify_signatures(&state,src_net_id, conn_sn, msg, signatures);
        state
    } 

}