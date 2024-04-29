#[test_only]
module xcall::xcall_tests {
    use sui::test_scenario::{Self, Scenario};
    use sui::coin::{Self};
    use xcall::main::{Self};
    use xcall::xcall_state::{Self, AdminCap, Storage };
    use std::string::{Self};
    use sui::sui::SUI;
    use xcall::network_address::{Self};
    use xcall::envelope::{Self};
    use xcall::message_request::{Self};
    use xcall::cs_message::{Self};


        #[test_only]
    fun setup_test(admin:address):Scenario {
        let mut scenario = test_scenario::begin(admin);
         scenario.next_tx(admin);
         scenario=main::init_xcall_state(admin,scenario);
         scenario
    }

    //   #[test]
    // fun test_setter_getter() {
    //     let admin = @0xBABE;

    //     let mut _scenario = setup_test(admin);
    //     let scenario = & _scenario;

    //     {
    //         let adminCap = test_scenario::take_from_sender<AdminCap>(scenario);
    //         let mut storage = test_scenario::take_shared<Storage>(scenario);
            
    //         main::set_protocol_fee(&mut storage, &adminCap, 100);
    //         assert!(xcall_state::get_protocol_fee(&storage) == 100, 1);

    //         main::set_protocol_fee_handler(&mut storage, &adminCap, @0xBABE);
    //         assert!(xcall_state::get_protocol_fee_handler(&storage) == @0xBABE, 1);


    //         test_scenario::return_to_sender(scenario, adminCap);
    //         test_scenario::return_shared( storage);
    //     };
    //     _scenario.end();
    // }


    #[test]
    fun test_send_call_message() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);

        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, ctx);
            let sources = vector[string::utf8(b"xcall"), string::utf8(b"connection")];
            let destinations = vector[string::utf8(b"icon:hx234"), string::utf8(b"icon:hx334")];
            let mut fee = coin::mint_for_testing<SUI>(100, ctx);
            let data = b"data";
            let envelope=envelope::wrap_call_message(data,sources,destinations);
            let envelope_bytes=envelope::encode(&envelope);
            main::send_call(&mut storage,&mut fee,&idCap,string::utf8(b"dnetId/daddress"),envelope_bytes,ctx);
            xcall_state::delete_id_cap_for_testing(idCap, ctx);
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);
            test_scenario::return_to_sender(&scenario, fee);
        };
        
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 1, 0);
        };
        test_scenario::end(scenario);
    }

    #[test]
    fun test_handle_message() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);

        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, ctx);
            let sources = vector[string::utf8(b"centralized")];
            let data = b"data";
            let dst_dapp = string::utf8(b"dsui/daddress");
            let src_dapp = network_address::create(string::utf8(b"icon"), string::utf8(b"address"));

            let from_nid = string::utf8(b"icon");
            let request = message_request::create(src_dapp, dst_dapp, 1, 1, data, sources);
            let message = cs_message::encode(&cs_message::new(cs_message::request_code(), message_request::encode(&request)));
            let conn_cap = xcall_state::create_conn_cap_for_testing(&mut storage);

            main::handle_message(&mut storage,&conn_cap,from_nid,message,ctx);
            // main::send_call(&mut storage,&mut fee,&idCap,string::utf8(b"dnetId/daddress"),envelope_bytes,ctx);
            xcall_state::delete_id_cap_for_testing(idCap, ctx);
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);
        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 1, 0);
        };
        test_scenario::end(scenario);
    }
  
     #[test]
    fun test_execute_call() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);

        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let sources = vector[string::utf8(b"centralized")];
            let data = b"data";
            let dst_dapp = string::utf8(b"dsui/daddress");
            let src_dapp = network_address::create(string::utf8(b"icon"), string::utf8(b"address"));

            let from_nid = string::utf8(b"icon");
            let request = message_request::create(src_dapp, dst_dapp, 1, 0, data, sources);
            let message = cs_message::encode(&cs_message::new(cs_message::request_code(), message_request::encode(&request)));
            let conn_cap = xcall_state::create_conn_cap_for_testing(&mut storage);

            main::handle_message(&mut storage,&conn_cap,from_nid,message,ctx);
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);
        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 1, 0);
        };
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, ctx);
            let mut fee = coin::mint_for_testing<SUI>(100, ctx);
            let data = b"data";

            let ticket = main::execute_call(&mut storage,&idCap,1,data, ctx);

            main::execute_call_result(&mut storage,ticket, true, &mut fee,ctx);
            xcall_state::delete_id_cap_for_testing(idCap, ctx);
            test_scenario::return_shared( storage);
            test_scenario::return_to_sender(&scenario, fee);

        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 1, 0);
        };
        test_scenario::end(scenario);
    }

}