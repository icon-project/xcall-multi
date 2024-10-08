module xcall::cluster_state {
    use std::string::{String};
    use sui::vec_map::{Self, VecMap};
    use xcall::xcall_utils::{Self as utils};
    use xcall::signatures::{pubkey_to_sui_address, verify_signature, get_pubkey_from_signature};
    use sui::coin::{Self};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    use sui::bag::{Bag, Self};
    use sui::event;

    //ERRORS
    const VerifiedSignaturesLessThanThreshold: u64 = 100;
    const NotEnoughSignatures: u64 = 101;
    const InvalidThreshold: u64 = 102;
    const RemovingValidatorNotInList: u64 = 103;
    const AdminValidatorCannotBeRemoved: u64 = 104;
    const ValidatorCountMustBeGreaterThanThreshold: u64 = 105;
    const ValidatorAlreadyExists: u64 = 106;

    //EVENTS
    public struct ValidatorAdded has copy, drop {
        validator: address
    }

    public struct ValidatorRemoved has copy, drop {
        validator: address
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

    public struct Validator has store, drop, copy{
        pub_key:vector<u8>,
        sui_address:address,
    }

    public struct State has store{ 
        message_fee: VecMap<String, u64>,
        response_fee: VecMap<String, u64>,
        receipts: VecMap<ReceiptKey, bool>,
        conn_sn: u128,
        balance: Balance<SUI>,
        validators: vector<Validator>,
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

    fun get_validator(pub_key:String):Validator{
        let (pubkey,sui_address)=pubkey_to_sui_address(&pub_key);
        Validator{
            pub_key:pubkey,
            sui_address:sui_address
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

    public(package) fun verify_signatures(self:&State,msg:vector<u8>,signatures:vector<vector<u8>>){
        let threshold=self.get_validator_threshold();
        let validators=self.get_validators().map!(|validator| validator.pub_key);
        assert!(signatures.length() >= threshold, NotEnoughSignatures);
        let mut i = 0;
        let mut unique_verified_pubkey = vector::empty();
        while (i < signatures.length()) {
            let signature = signatures.borrow(i);
            let pub_key = get_pubkey_from_signature(signature);
            if (validators.contains(&pub_key)) {
                
                if (verify_signature(&pub_key,signature,&msg)){

                    if (!unique_verified_pubkey.contains(&pub_key)){
                        unique_verified_pubkey.push_back(pub_key);
                    };

                    if (unique_verified_pubkey.length() >= threshold) {
                        return
                    };
                };
            };
            i=i+1;
        };
        assert!(unique_verified_pubkey.length() >= threshold, VerifiedSignaturesLessThanThreshold); 
    }

    public(package) fun get_validator_threshold(self:&State):u64{
        self.validators_threshold
    }

    public(package) fun set_validator_threshold(self:&mut State,threshold:u64){
        assert!(threshold <= self.validators.length(), InvalidThreshold);
        self.validators_threshold=threshold
    }

    public(package) fun add_validator(self:&mut State,validator_pub_key:String){
        let validator=get_validator(validator_pub_key);
        assert!(!self.validators.contains(&validator), ValidatorAlreadyExists);
        self.validators.push_back(validator);
        event::emit(ValidatorAdded { validator: validator.sui_address });
    }

    public(package) fun remove_validator(self:&mut State,validator_pub_key:String,ctx:&TxContext){
        assert!(self.validators.length() > self.validators_threshold, ValidatorCountMustBeGreaterThanThreshold);
        let validator=get_validator(validator_pub_key);
        let (contains, index) = self.validators.index_of(&validator);
        assert!(contains, RemovingValidatorNotInList);
        assert!(ctx.sender() != validator.sui_address, AdminValidatorCannotBeRemoved);
        self.validators.remove(index);

        event::emit(ValidatorRemoved { validator: validator.sui_address });
    }

    public(package) fun get_validators(self:&State):vector<Validator>{
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
    use xcall::cluster_state::{State, get_validators, get_validator_threshold, set_validator_threshold, add_validator, remove_validator, verify_signatures};
    use sui::test_scenario::{Self, Scenario};

    #[test]
    fun test_add_validator(): State {
        let mut state = xcall::cluster_state::create_state();
        add_validator(&mut state, b"AJ6snNNaDhPZLg06AkcvYL0TZe4+JgoWtZKG/EJmzdWi".to_string());
        add_validator(&mut state, b"ADDxHCpQUcFsy5H5Gy01uv7LoISvtJLfgVGfWy4bLrjO".to_string());
        add_validator(&mut state, b"AL0hUNIiz5Q2fv0siZc75ce3aOyUpiiI+Q8Rmfay4K/X".to_string());
        add_validator(&mut state, b"ALnG7hYw7z5xEUSmSNsGu7IoT3J0z77lP/zuUDzBpJIA".to_string());

        let validators = get_validators(&state);
        assert!((validators.length() == 4));

        assert!(get_validator_threshold(&state)==0);

        set_validator_threshold(&mut state, 2);
        assert!(get_validator_threshold(&state)==2);

        state
    }

    #[test]
    #[expected_failure(abort_code = 106)]
    fun test_add_repeated_validator(): State {
        let mut state = test_add_validator();
        add_validator(&mut state, b"AJ6snNNaDhPZLg06AkcvYL0TZe4+JgoWtZKG/EJmzdWi".to_string());
        add_validator(&mut state, b"AJ6snNNaDhPZLg06AkcvYL0TZe4+JgoWtZKG/EJmzdWi".to_string());
        state
    }

    #[test]
    fun test_set_get_threshold(): State {
        let mut state = test_add_validator();
        set_validator_threshold(&mut state, 2);
        assert!(get_validator_threshold(&state)==2);
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
    #[expected_failure(abort_code = 103)]
    fun test_remove_validator_not_in_list(): State {
        let mut scenario = test_scenario::begin(@0xadd);
        let mut state = test_set_get_threshold();
        remove_validator(&mut state, b"ALnG7hYw7z5xEUSmSNsGu7IoT3J0z77lS/zuUDzBpJIA".to_string(), scenario.ctx());
        test_scenario::end(scenario);
        state
    }

    #[test]
    fun test_remove_validator(): State {
        let mut scenario = test_scenario::begin(@0xadd);
        let mut state = test_set_get_threshold();
        remove_validator(&mut state, b"AJ6snNNaDhPZLg06AkcvYL0TZe4+JgoWtZKG/EJmzdWi".to_string(), scenario.ctx());
        test_scenario::end(scenario);

        let validators = get_validators(&state);
        assert!((validators.length() == 3));
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
        let state = test_set_get_threshold();
        
        let msg: vector<u8> = x"6162636465666768";
        let signatures = vector[x"00bb0a7ba4a242a4988c820b94a8df9b312e9e7cf4f8302b53ee2e046e76da86eae9c15296a421b8dddb29cafa8d50523e0b04300216e393d45c0739a0eab8e60cb9c6ee1630ef3e711144a648db06bbb2284f7274cfbee53ffcee503cc1a49200"]; 

        xcall::cluster_state::verify_signatures(&state, msg, signatures);
        state
    }

    #[test]
    #[expected_failure(abort_code = 100)]
    fun test_verify_signatures_invalid(): State {
        let state = test_set_get_threshold();
        let msg: vector<u8> = x"6162636465666768";
        let signatures = vector[x"00bb0a7ba4a242a4988c820b94a8df9b312e9e7cf4f8302b53ee2e046e76da86eae9c15296a421b8dddb29cafa8d50523e0b04300216e393d45c0739a0eab8e60cb9c6ee1630ef3e711144a648db06bbb2284f7274cfbee53ffcee503cc1a49200",
                                                    x"00c6d94cc625e73e036852316d228e578893aad2a7b21febc08f92a5d53978154782e4a0551ced23fb92c765b4cb4715e231de0235e2b641b81a36b9f2a3f8630d9eac9cd35a0e13d92e0d3a02472f60bd1365ee3e260a16b59286fc4266cdd5a2"
                                                   ];

        xcall::cluster_state::verify_signatures(&state, msg, signatures);
        state
    }

    #[test]
    fun test_verify_signatures(): State {
        let state = test_set_get_threshold();
        let msg: vector<u8> = x"6162636465666768";
        let signatures = vector[x"00bb0a7ba4a242a4988c820b94a8df9b312e9e7cf4f8302b53ee2e046e76da86eae9c15296a421b8dddb29cafa8d50523e0b04300216e393d45c0739a0eab8e60cb9c6ee1630ef3e711144a648db06bbb2284f7274cfbee53ffcee503cc1a49200",
                                                    x"00c6d94cc625e73e036852316d229e578893aad2a7b21febc08f92a5d53978154782e4a0551ced23fb92c765b4cb4715e231de0235e2b641b81a36b9f2a3f8630d9eac9cd35a0e13d92e0d3a02472f60bd1365ee3e260a16b59286fc4266cdd5a2"
                                                   ];

        xcall::cluster_state::verify_signatures(&state, msg, signatures);
        state
    } 

}