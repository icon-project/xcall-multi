#[test_only]
module xcall::xcall_tests {
    use sui::test_scenario::{Self, Scenario};
    use sui::coin::{Self};
    use xcall::main::{Self};
    use xcall::xcall_state::{Self, AdminCap, Storage,ConnCap};
    use std::string::{Self, String};
    use sui::sui::SUI;
    use xcall::network_address::{Self};
    use xcall::envelope::{Self};
    use xcall::message_request::{Self};
    use xcall::message_result::{Self};
    use xcall::cs_message::{Self};
    use xcall::centralized_entry;
    use xcall::xcall_utils as utils;
    use sui::package::UpgradeCap;

    #[test_only]
    fun setup_test(admin:address):Scenario {
        let mut scenario = test_scenario::begin(admin);
         scenario.next_tx(admin);
         scenario=main::init_xcall_state(admin,scenario);
         scenario
    }

    #[test_only]
    fun setup_connection(mut scenario_val: Scenario, from_nid: String, admin:address): Scenario {
        let scenario = &mut scenario_val;
        {
            let mut storage = test_scenario::take_shared<Storage>(scenario);
             let adminCap = scenario.take_from_sender<AdminCap>();
            main::register_connection_admin(&mut storage, &adminCap, string::utf8(b"centralized-1"),admin, scenario.ctx());
            main::register_connection_admin(&mut storage, &adminCap, string::utf8(b"centralized-2"),admin, scenario.ctx());
            main::set_default_connection(&mut storage, &adminCap,from_nid, string::utf8(b"centralized-2"), scenario.ctx());

            test_scenario::return_shared(storage);
            scenario.return_to_sender(adminCap);
        };
        test_scenario::next_tx(scenario, admin);
        scenario_val
    }

    #[test_only]
    fun setup_nid(mut scenario_val: Scenario, nid: String, admin:address): Scenario {
        let scenario = &mut scenario_val;
        {
            let mut storage = test_scenario::take_shared<Storage>(scenario);
            let adminCap = scenario.take_from_sender<AdminCap>();
            main::configure_nid_test(&mut storage, &adminCap,nid, scenario.ctx());
            test_scenario::return_shared(storage);
            scenario.return_to_sender(adminCap);
        };
        test_scenario::next_tx(scenario, admin);
        scenario_val
    }

    #[test]
    fun test_setter_getter() {
        let admin = @0xBABE;

        let mut _scenario = setup_test(admin);
        let scenario = & _scenario;

        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(scenario);
            let mut storage = test_scenario::take_shared<Storage>(scenario);
            
            main::set_protocol_fee(&mut storage, &adminCap, 100);
            assert!(xcall_state::get_protocol_fee(&storage) == 100, 1);

            main::set_protocol_fee_handler(&mut storage, &adminCap, @0xBABE);
            assert!(xcall_state::get_protocol_fee_handler(&storage) == @0xBABE, 1);


            test_scenario::return_to_sender(scenario, adminCap);
            test_scenario::return_shared( storage);
        };
        _scenario.end();
    }


    #[test]
    fun test_send_call_message() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);

        test_scenario::next_tx(&mut scenario, admin);

        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);
        
        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, ctx);
            let sources = vector[string::utf8(b"centralized-1")];
            let destinations = vector[string::utf8(b"icon:hx234")];
            let fee = coin::mint_for_testing<SUI>(100, ctx);
            let data = b"This implementation ensures that the RLP-encoded list is correctly decoded, including handling empty elements. The utility functions help manage byte operations and simplify the main decoding logic.";
            let envelope=envelope::wrap_call_message(data,sources,destinations);
            let envelope_bytes=envelope::encode(&envelope);
            main::send_call(&mut storage,fee,&idCap,string::utf8(b"dnetId/daddress"),envelope_bytes,ctx);
            xcall_state::delete_id_cap_for_testing(idCap, ctx);
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);
        };
        
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 2, 0);
        };
        test_scenario::end(scenario);
    }

     #[test]
    fun test_set_fee_claim_fee() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);

        test_scenario::next_tx(&mut scenario, admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);
        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);
           let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
            centralized_entry::set_fee(&mut storage,&conn_cap, b"icon".to_string(),50, 50, scenario.ctx());
            let fee = centralized_entry::get_fee(&mut storage,conn_cap.connection_id(), b"icon".to_string(),true, scenario.ctx());

            assert!(fee == 100, 3);
             scenario.return_to_sender(conn_cap);

            test_scenario::return_shared( storage);

        };
        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, ctx);
            let sources = vector[string::utf8(b"centralized-1")];
            let destinations = vector[string::utf8(b"icon:hx234")];
            let fee = coin::mint_for_testing<SUI>(100, ctx);
            let data = b"data";
            let envelope=envelope::wrap_call_message(data,sources,destinations);
            let envelope_bytes=envelope::encode(&envelope);
            main::send_call(&mut storage,fee,&idCap,string::utf8(b"icon/address"),envelope_bytes,ctx);
            xcall_state::delete_id_cap_for_testing(idCap, ctx);
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);
        };
        test_scenario::next_tx(&mut scenario, admin);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);
            let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
            centralized_entry::claim_fees(&mut storage,&conn_cap, scenario.ctx());
             scenario.return_to_sender(conn_cap);
            test_scenario::return_shared( storage);
        };
        
        test_scenario::end(scenario);
    }

    #[test]
    fun test_handle_message() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);
scenario = setup_connection(scenario, string::utf8(b"icon"), admin);
        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, scenario.ctx());
            let sources = vector[string::utf8(b"centralized-1")];
            let data = b"data";
            let sui_dapp = string::utf8(b"dsui/daddress");
            let icon_dapp = network_address::create(string::utf8(b"icon"), string::utf8(b"address"));

            let from_nid = string::utf8(b"icon");
            let request = message_request::create(icon_dapp, sui_dapp, 1, 1, data, sources);
            let message = cs_message::encode(&cs_message::new(cs_message::request_code(), message_request::encode(&request)));
            let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
            main::handle_message(&mut storage,&conn_cap,from_nid,message,scenario.ctx());
            xcall_state::delete_id_cap_for_testing(idCap, scenario.ctx());
            test_scenario::return_to_sender(&scenario, adminCap);
             scenario.return_to_sender(conn_cap);
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
    fun test_handle_message_response() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);

        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);

        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, scenario.ctx());

            let data = b"data";
            let rollback_data = b"rollback";
             // since we are registering 2 connections take_from_sender returns latest one.
            let sources = vector[string::utf8(b"centralized-2")];
            let destinations = vector[string::utf8(b"icon")];
            let fee = coin::mint_for_testing<SUI>(100, scenario.ctx());

            let envelope=envelope::wrap_call_message_rollback(data,rollback_data,sources,destinations);
            let envelope_bytes=envelope::encode(&envelope);
            main::send_call(&mut storage,fee,&idCap,string::utf8(b"dnetId/daddress"),envelope_bytes,scenario.ctx());

            xcall_state::delete_id_cap_for_testing(idCap, scenario.ctx());
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);

        };
         test_scenario::next_tx(&mut scenario, admin);

        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
         //   let ctx = test_scenario::ctx(&mut scenario);

            let response = message_result::create(1, message_result::failure(),b"");
            let message = cs_message::encode(&cs_message::new(cs_message::result_code(), message_result::encode(&response)));
            let from_nid = string::utf8(b"icon");
           let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
            main::handle_message(&mut storage,&conn_cap,from_nid,message,scenario.ctx());
             scenario.return_to_sender(conn_cap);
            test_scenario::return_shared( storage);
        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 2, 0);
        };
        test_scenario::end(scenario);
    }
  
     #[test]
    fun test_execute_call() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);
        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);

        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            //let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, scenario.ctx());
            let sources = vector[string::utf8(b"centralized")];
            let data = b"data";
            let mut sui_dapp = b"".to_string();
            sui_dapp.append(utils::id_to_hex_string(&xcall_state::get_id_cap_id(&idCap)));
            let icon_dapp = network_address::create(string::utf8(b"icon"), string::utf8(b"address"));

            let from_nid = string::utf8(b"icon");
            let request = message_request::create(icon_dapp, sui_dapp, 1, 0, data, sources);
            let message = cs_message::encode(&cs_message::new(cs_message::request_code(), message_request::encode(&request)));
             let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
            main::handle_message(&mut storage,&conn_cap,from_nid,message,scenario.ctx());

            let fee = coin::mint_for_testing<SUI>(100, scenario.ctx());
            let data = b"data";

            let ticket = main::execute_call(&mut storage,&idCap,1,data, scenario.ctx());

            main::execute_call_result(&mut storage,ticket, true,fee,scenario.ctx());
            xcall_state::delete_id_cap_for_testing(idCap, scenario.ctx());
            test_scenario::return_to_sender(&scenario, adminCap);
             scenario.return_to_sender(conn_cap);
            test_scenario::return_shared( storage);
        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 2, 0);
        };
        test_scenario::end(scenario);
    }

    #[test]
    fun test_execute_rollback() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);

        test_scenario::next_tx(&mut scenario, admin);
        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);

        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, scenario.ctx());

            let data = b"data";
            let rollback_data = b"rollback";

            let sources = vector[string::utf8(b"centralized-2")];
            let destinations = vector[string::utf8(b"icon")];
            let fee = coin::mint_for_testing<SUI>(100, scenario.ctx());

            let envelope=envelope::wrap_call_message_rollback(data,rollback_data,sources,destinations);
            let envelope_bytes=envelope::encode(&envelope);
            main::send_call(&mut storage,fee,&idCap,string::utf8(b"dnetId/daddress"),envelope_bytes,scenario.ctx());

            let response = message_result::create(1, message_result::failure(),b"");
            let message = cs_message::encode(&cs_message::new(cs_message::result_code(), message_result::encode(&response)));
            let from_nid = string::utf8(b"icon");
             let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
             std::debug::print(&conn_cap);

            main::handle_message(&mut storage,&conn_cap,from_nid,message,scenario.ctx());

            let ticket = main::execute_rollback(&mut storage,&idCap,1, scenario.ctx());

            main::execute_rollback_result(&mut storage,ticket,true);

            xcall_state::delete_id_cap_for_testing(idCap, scenario.ctx());
            test_scenario::return_to_sender(&scenario, adminCap);
            scenario.return_to_sender(conn_cap);
            test_scenario::return_shared( storage);
        };

        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            
            assert!(events == 5, 0);
        };
       
        test_scenario::end(scenario);
    }


     #[test]
    fun test_handle_reply() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);

        test_scenario::next_tx(&mut scenario, admin);
        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);

        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, ctx);

            let data = b"data";
            let rollback_data = b"data";

            let sources = vector[string::utf8(b"centralized-1")];
            let destinations = vector[string::utf8(b"icon")];
            let fee = coin::mint_for_testing<SUI>(100, ctx);

            let icon_dapp = string::utf8(b"icon/address");

            let envelope=envelope::wrap_call_message_rollback(data,rollback_data,sources,destinations);
            let envelope_bytes=envelope::encode(&envelope);
            main::send_call(&mut storage,fee,&idCap,icon_dapp,envelope_bytes,ctx);

            xcall_state::delete_id_cap_for_testing(idCap, ctx);
            test_scenario::return_to_sender(&scenario, adminCap);
            test_scenario::return_shared( storage);

        };
        test_scenario::next_tx(&mut scenario, admin);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);

            let sources = vector[string::utf8(b"centralized")];
            let data = b"data";
            let sui_dapp = string::utf8(b"dsui/daddress");
            let icon_dapp = network_address::create(string::utf8(b"icon"), string::utf8(b"address"));

            let request = message_request::create(icon_dapp, sui_dapp, 1, 1, data, sources);
            let response = message_result::create(1, message_result::success(),request.encode());
            let message = cs_message::encode(&cs_message::new(cs_message::result_code(), response.encode()));
            let from_nid = string::utf8(b"icon");
            let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);

            main::handle_message(&mut storage,&conn_cap,from_nid,message,scenario.ctx());
            scenario.return_to_sender(conn_cap);
            test_scenario::return_shared( storage);
        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 2, 0);
        };
        
        test_scenario::end(scenario);
    }

        #[test]
    fun test_handle_message_multi_protocols() {
        let admin = @0xBABE;

        let mut scenario = setup_test(admin);
        scenario = setup_nid(scenario, string::utf8(b"sui"), admin);
        scenario = setup_connection(scenario, string::utf8(b"icon"), admin);
        test_scenario::next_tx(&mut scenario, admin);
        {
            let adminCap = test_scenario::take_from_sender<AdminCap>(&scenario);
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
           // let ctx = test_scenario::ctx(&mut scenario);
            let idCap = xcall_state::create_id_cap(&storage, scenario.ctx());
            let sources = vector[string::utf8(b"centralized-1"), string::utf8(b"centralized-2")];
            let data = b"data";
            let sui_dapp = string::utf8(b"dsui/daddress");
            let icon_dapp = network_address::create(string::utf8(b"icon"), string::utf8(b"address"));

            let from_nid = string::utf8(b"icon");
            let request = message_request::create(icon_dapp, sui_dapp, 1, 1, data, sources);
            let message = cs_message::encode(&cs_message::new(cs_message::request_code(), message_request::encode(&request)));
            let conn_cap = test_scenario::take_from_sender<ConnCap>(&scenario);
            main::handle_message(&mut storage,&conn_cap,from_nid,message,scenario.ctx());
            
            xcall_state::delete_id_cap_for_testing(idCap, scenario.ctx());
            test_scenario::return_to_sender(&scenario, adminCap);
             scenario.return_to_sender(conn_cap);
            test_scenario::return_shared( storage);
        };  
        {
            let abc = test_scenario::next_tx(&mut scenario, admin);
            let events = test_scenario::num_user_events(&abc);
            assert!(events == 0, 0);
        };
        test_scenario::end(scenario);
    }

}