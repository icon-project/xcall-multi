
#[test_only]
module multisig::multisig_tests {
    // uncomment this line to import the module
    use multisig::multisig;
      use sui::test_scenario::{Self, Scenario};
      use multisig::multisig::AdminCap;
      use multisig::multisig::Storage;
      use std::string::{Self,String};
      use multisig::base64::{Self};

    const ENotImplemented: u64 = 0;

    #[test_only]
    fun setup_test(admin:address):Scenario {
        let mut scenario = test_scenario::begin(admin);
        scenario.next_tx(admin);
        scenario=multisig::init_state(admin,scenario);
        scenario
    }

     #[test_only]
    fun get_test_pub_keys():vector<String>{
         let mut pubkeys:vector<String> =vector::empty();
            pubkeys.push_back(string::utf8(b"AQM6YkAASHEsBpZFbeiCwm0Rmj3y/jFsWrFzi6kHJhJoFA=="));
            pubkeys.push_back(string::utf8(b"AAFuArenKCbZUXiXkfPAU/kq+bwosnqOLGD8R0ylNvSB"));
            pubkeys.push_back(string::utf8(b"AQNBlbWmHu6+5v0rlZ724j9zk7KwcXtkWO6+b/cneNY4BA=="));
            pubkeys.push_back(string::utf8(b"AgI+y4/2z460dI72+wYutS+GKwk+s6QtYp2J5D2WRRCPng=="));
            pubkeys.push_back(string::utf8(b"AQOxMDbUpK33ucNsXNFlYTyCc051QeOkeGT7W4cn55IFmA=="));
            pubkeys

    }

    fun get_test_weights():vector<u8>{
        let mut weights: vector<u8> = vector::empty();
            weights.push_back(1);
            weights.push_back(1);
            weights.push_back(1);
            weights.push_back(1);
            weights.push_back(1);
        weights
    }

    #[test]
   fun test_register_wallet() {
     let admin = @0xBABE;

        let mut _scenario = setup_test(admin);
        let scenario = & _scenario;

        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(scenario);
            let mut storage = test_scenario::take_shared<Storage>(scenario);
            // storage:&mut Storage,_admin:&AdminCap, pub_keys:vector<String>,weights:vector<u8>,threshold:u16
            let  pubkeys:vector<String> =get_test_pub_keys();
            let  weights: vector<u8> = get_test_weights();
            let expected= @0x34f45f30d3af0393474ce42fc7a1de48aa8a9ddf03383062d8fcd1842d627a2f;


            multisig::register_wallet(&mut storage,&adminCap,pubkeys,weights,3);
            assert!(storage.get_wallets().size()>0);
            let wallet=storage.get_wallets().get(&expected);
            assert!(wallet.multisig_address()==expected);
            scenario.return_to_sender(adminCap);
            test_scenario::return_shared( storage);
        };
        _scenario.end();
    
}

#[test]
fun test_create_proposal() {

     let admin = @0x29a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c204589;

        let mut scenario = setup_test(admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            // storage:&mut Storage,_admin:&AdminCap, pub_keys:vector<String>,weights:vector<u8>,threshold:u16
            let  pubkeys:vector<String> =get_test_pub_keys();
            let  weights: vector<u8> = get_test_weights();
          
            multisig::register_wallet(&mut storage,&adminCap,pubkeys,weights,3);
            
            
            scenario.return_to_sender(adminCap);
            test_scenario::return_shared( storage);
          
        };
        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let expected= @0x34f45f30d3af0393474ce42fc7a1de48aa8a9ddf03383062d8fcd1842d627a2f;
            let tx_data=x"00000200203fab45fb191ca013a74ccfc3b7d5ed27a3ef6dce79adc4d1e39555f01a361bf801001b61730f57e4d64241cecb40ac259c58ced75018bd231acdbe463ddb9ac99176480000000000000020e12acf2025db82d420e8696d3645dfd25c6542382883f6e3762cc655791a007e01010101010001000029a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c20458901c2a6df77449ebce38397d97785f52c2341abafc21c47523e9cfb71d141d5df614800000000000000209e4e3d3f48881797ecc3690b237e0353e16a8be1cd79ac75210657229942e12b29a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c204589e80300000000000078be2d000000000000";
            let tx_data_64=base64::encode(&tx_data);
            multisig::create_proposal(&mut storage,string::utf8(b"test proposal"),tx_data_64,expected,scenario.ctx());
            assert!(storage.get_proposals().length()>0);
            scenario.return_to_sender(adminCap);
            test_scenario::return_shared( storage);

        };

        scenario.end();
   
}

#[test]
fun test_approve_proposal() {

      let admin = @0x29a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c204589;

        let mut scenario = setup_test(admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            // storage:&mut Storage,_admin:&AdminCap, pub_keys:vector<String>,weights:vector<u8>,threshold:u16
            let  pubkeys:vector<String> =get_test_pub_keys();
            let  weights: vector<u8> = get_test_weights();
            let expected= @0x34f45f30d3af0393474ce42fc7a1de48aa8a9ddf03383062d8fcd1842d627a2f;
            let tx_data=x"00000200203fab45fb191ca013a74ccfc3b7d5ed27a3ef6dce79adc4d1e39555f01a361bf801001b61730f57e4d64241cecb40ac259c58ced75018bd231acdbe463ddb9ac99176480000000000000020e12acf2025db82d420e8696d3645dfd25c6542382883f6e3762cc655791a007e01010101010001000029a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c20458901c2a6df77449ebce38397d97785f52c2341abafc21c47523e9cfb71d141d5df614800000000000000209e4e3d3f48881797ecc3690b237e0353e16a8be1cd79ac75210657229942e12b29a0918bee7a7e37d1a7d0613efc3f4455883ea217046f7db91d53e69c204589e80300000000000078be2d000000000000";
            let tx_data_64=base64::encode(&tx_data);
            multisig::register_wallet(&mut storage,&adminCap,pubkeys,weights,3);
            test_scenario::next_tx(&mut scenario, admin);
            multisig::create_proposal(&mut storage,string::utf8(b"test proposal"),tx_data_64,expected,scenario.ctx());
            test_scenario::next_tx(&mut scenario, admin);
            assert!(storage.get_proposals().length()>0);
            let signature=x"0196e3d1a05e3d9d900281da7a3719dada72b66fdcfd4147275634f3028d71dba02896bf6958bdc97d93462bd0aa7245a04f05caa3e7f09465d725fa79f91fcc76033a62400048712c0696456de882c26d119a3df2fe316c5ab1738ba90726126814";
            let signature_64=base64::encode(&signature);
            multisig::approve_proposal(&mut storage,1,signature_64,scenario.ctx());
            scenario.return_to_sender(adminCap);
            test_scenario::return_shared( storage);
          
        };
        scenario.end();
  
}
}

