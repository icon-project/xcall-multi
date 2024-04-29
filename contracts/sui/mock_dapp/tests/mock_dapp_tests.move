
#[test_only]
module mock_dapp::mock_dapp_tests {
    use mock_dapp::mock_dapp::{init};
    use sui::test_scenario::{Self,Scenario};
    use std::debug;
    use xcall::main::{Self as xcall,init_xcall_state};
    use mock_dapp::dapp_state::{Self,DappState};
    use mock_dapp::mock_dapp::{Self as mock_dapp,WitnessCarrier};
    use xcall::xcall_state::{Self,Storage as XCallState};
    use std::string::{Self,String};
    use xcall::centralized_entry::{Self as connection_in};
    use xcall::call_message::{Self};
    use xcall::envelope::{Self};
    use xcall::message_request::{Self};
    use xcall::network_address::{Self};
    use xcall::cs_message::{Self};
    use std::bcs::{Self};

    

    
    #[test_only]
    fun setup_test(admin:address):Scenario {
        let mut scenario = test_scenario::begin(admin);
         scenario= mock_dapp::init_dapp_state(admin,scenario);
         scenario.next_tx(admin);
         scenario=init_xcall_state(admin,scenario);
         scenario
    }

    #[test_only]
    fun setup_register_xcall(admin:address,mut scenario:Scenario):Scenario{
        let carrier = scenario.take_from_sender<WitnessCarrier>();
        let xcall_state= scenario.take_shared<XCallState>();
        scenario.next_tx(admin);
        mock_dapp::register_xcall(&xcall_state,carrier,scenario.ctx());
        test_scenario::return_shared<XCallState>(xcall_state);
        scenario

    }

    #[test_only]
    fun setup_connection(admin:address,mut scenario:Scenario):Scenario {
        let mut xcall_state= scenario.take_shared<XCallState>();
        xcall::register_connection(&mut xcall_state,string::utf8(b"netid"),string::utf8(b"centralized"),scenario.ctx());
        test_scenario::return_shared<XCallState>(xcall_state);
        scenario.next_tx(admin);
        let mut dapp_state=scenario.take_shared<DappState>();
        mock_dapp::add_connection(&mut dapp_state,string::utf8(b"netid"),string::utf8(b"centralized"),string::utf8(b"destconn"),scenario.ctx());
        test_scenario::return_shared<DappState>(dapp_state);

        scenario



    }

    



    #[test]
    fun test_register_xcall(){
        let admin = @0xAD;
        let sender = @0xCAFE;
        let mut scenario= setup_test(admin);
        scenario= setup_register_xcall(admin,scenario);
        scenario.next_tx(admin);
        let dapp_state=scenario.take_shared<DappState>();
        debug::print(&dapp_state);
        test_scenario::return_shared<DappState>(dapp_state);
        scenario.end();

    }

    #[test_only]
    fun create_message_request_payload(msg:vector<u8>,to:String):vector<u8>{
        let mut sources=vector::empty<String>();
        sources.push_back(string::utf8(b"centralized"));
        let from =network_address::from_string(string::utf8(b"dnetId/daddress"));
        let request= message_request::create(from,to,21,0,msg,sources);
        let cs_message=cs_message::from_message_request(request);
        let bytes=cs_message.encode();
        bytes
        
    }

    #[test]
    fun test_receive_message(){
        let admin = @0xAD;
        let sender = @0xCAFE;
        let mut scenario= setup_test(admin);
        scenario= setup_register_xcall(admin,scenario);
        scenario.next_tx(admin);
        scenario= setup_connection(admin,scenario);
        scenario.next_tx(admin);
        let dapp_state=scenario.take_shared<DappState>();
        let mut xcall_state= scenario.take_shared<XCallState>();
        debug::print(&xcall_state);
        let payload= create_message_request_payload(b"somedata",dapp_state.id_str());
        connection_in::receive_message(&mut xcall_state,string::utf8(b"source"),1,payload,scenario.ctx());
        debug::print(&dapp_state);
        test_scenario::return_shared<DappState>(dapp_state);
        test_scenario::return_shared<XCallState>(xcall_state);
        scenario.end();

    }




    

    
}

