#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::main {

    // Part 1: Imports
    use sui::object::{Self, UID,ID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::linked_table::{Self, LinkedTable};
    use sui::types as sui_types;
    use std::string::{Self, String};
    use std::vector::{Self};
    use std::option::{Self, Option};
    use sui::event;
    use sui::hash::{Self};
     use sui::coin::{Self,Coin};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
   
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::envelope::{Self,XCallEnvelope};
    use xcall::connections::{Self,register};
    use xcall::message_request::{Self, CSMessageRequest};
    use xcall::call_message_rollback::{Self,CallMessageWithRollback};
    use xcall::message_result::{Self,CSMessageResult};
    use xcall::cs_message::{Self};
    use xcall::rollback_data::{Self,RollbackData};
    use xcall::xcall_state::{Self,Storage,AdminCap,IDCap,ConnCap};
    use xcall::execute_ticket::{Self,ExecuteTicket};
    use xcall::rollback_ticket::{Self,RollbackTicket};
    use xcall::cs_message::{CSMessage};
    use sui::bag::{Bag, Self};
    use sui::table::{Table,Self};
    use sui::package::{Self,Publisher};
  
    use sui::vec_map::{Self, VecMap};
    use sui::versioned::{Self, Versioned};
   
     use sui::address::{Self};


    const ENotOneTimeWitness: u64 = 0;
    const ENotAdmin: u64 = 1;
    const ENotUpgrade: u64 = 2;
    const EWrongVersion: u64 = 3;
    const EInvalidNID: u64 = 4;
    const EInvalidSource: u64 = 5;
    const ENoRollback: u64 = 6;
    const EInvalidReply: u64 = 7;
    const EDataMismatch: u64 = 8;
    const EInvalidMsgType: u64 = 9;
    const ERollbackNotEnabled:u64 = 10;
    const EInfallible:u64 = 11;

    const CS_RESP_SUCCESS: u8 = 1;
    const CS_RESP_FAILURE: u8 = 0;

    const CALL_MESSAGE_TYPE: u8 = 0;
    const CALL_MESSAGE_ROLLBACK_TYPE: u8 = 1;
    const PERSISTENT_MESSAGE_TYPE: u8 = 2;

    const MAX_DATA_SIZE: u64 = 2048;

    const NID: vector<u8> = b"nid";

    const CURRENT_VERSION: u64 = 1;

    

    /*************Events*****************/

    public struct CallMessageSent has copy, drop{
        from:String,
        to:String,
        sn:u128,
    }

    public struct CallMessage has copy, drop{
        from:NetworkAddress,
        to:String,
        sn:u128,
        req_id:u128,
        data:vector<u8>,
    }

    public struct RollbackMessage has copy, drop{
        sn:u128
    }

    public struct RollbackExecuted has copy, drop{
        sn:u128
    }

    public struct ResponseMessage has copy, drop{
        sn:u128,
        response_code: u8
    }
    /***************/
    /******** tickets ******/
   
    
    fun init(ctx: &mut TxContext) {
        let admin = xcall_state::create_admin_cap(ctx);
        let storage = xcall_state::create_storage(
           CURRENT_VERSION,
            &admin,
            ctx
        );
       xcall_state::share(storage);
       xcall_state::transfer_admin_cap(admin,ctx);
    }

    #[test_only]
    public fun init_for_testing(ctx: &mut TxContext) {
        init(ctx)
    }

    public fun register_dapp<T: drop>(self:&Storage,
        witness: T,
        ctx: &mut TxContext
    ):IDCap {
        self.create_id_cap(ctx)
    }

    entry fun get_network_address(self: &mut Storage): network_address::NetworkAddress{
        xcall_state::network_address(self)
    }

    entry fun get_net_id(self: &mut Storage): String{
        string::utf8(NID)
    }

    entry public fun register_connection(self:&mut Storage,net_id:String,package_id:String,ctx: &mut TxContext){
        self.set_connection(net_id,package_id);
        let cap= xcall_state::new_conn_cap(self.get_id(),package_id);
        register(self.get_connection_states_mut(),package_id,cap,ctx);
    }

    public fun admin(self:&mut Storage):ID{
        xcall_state::get_admin(self)
    }

    public fun get_fee_handler(self:&mut Storage):address{
        xcall_state::get_protocol_fee_handler(self)
    }

    fun get_connection_fee(self:&mut Storage,connection:&String,net_id:String, rollback:bool ):u128{
        // connections::get_fee(self,connection,net_id,rollback)
        0
    }

    fun get_fee_connection_sn(self:&mut Storage,connection:&String,net_id:String, sn:u128 ):u128{
        // connections::get_fee(self,connection,net_id,sn>0)
        0
    }

    entry public fun get_fee(self:&mut Storage, net_id:String, rollback:bool):u128{
        // get_connection_fee(self, xcall_state::get_connection(self,net_id), net_id, rollback)
        0
    }

    entry fun get_fee_sources(self:&mut Storage, net_id:String, rollback:bool, sources:vector<String>):u128{
        let mut fee = xcall_state::get_protocol_fee(self);

        if(isReply(self,net_id,sources) && !rollback){
            return 0
        };

        let mut i = 0;
        while(i < vector::length(&sources)){
            let source = vector::borrow(&sources, i);
            fee = fee + get_connection_fee(self,source, net_id, rollback);
            i=i+1
        };

        fee
    }


    fun send_call_inner(self:&mut Storage,fee:&mut Coin<SUI>,from:NetworkAddress,to:NetworkAddress,envelope:XCallEnvelope,ctx: &mut TxContext){

        let sequence_no=get_next_sequence(self);
        let rollback=envelope::rollback(&envelope);
        let msg_type=envelope::msg_type(&envelope);

        let mut need_response = false;
        let data;

        if(msg_type == CALL_MESSAGE_TYPE || msg_type == PERSISTENT_MESSAGE_TYPE){
            data = envelope::message(&envelope);
        }
        else if(msg_type == CALL_MESSAGE_ROLLBACK_TYPE){
            let msg = call_message_rollback::decode(envelope::message(&envelope));
            let from_id = object::id_from_bytes(*string::bytes(&network_address::addr(&from)));

            let rollback = rollback_data::create(
                from_id,
                to.net_id(),
                envelope.sources(),
                msg.rollback(),
                false
            );

            xcall_state::add_rollback(self,sequence_no,rollback);

            need_response = true;
            data = call_message_rollback::data(&msg);
        }
        else{
            abort EInvalidMsgType
        };


        let dst_account = to.addr();

        let cs_request= message_request::create(
            from,
            dst_account,
            sequence_no,
            envelope::msg_type(&envelope),
            data,
            envelope::destinations(&envelope));

        let msg = message_request::encode(&cs_request);

        assert!(vector::length(&msg) <= MAX_DATA_SIZE, EInvalidReply);

        if(isReply(self,to.net_id(),envelope::sources(&envelope))){
            xcall_state::remove_reply_state(self);
            xcall_state::set_call_reply(self,msg);
        } else{
            let sendSn = if (need_response) {sequence_no} else {0};
            let cs_message=cs_message::from_message_request(cs_request);
            
       
            connection_send_message(self,
            fee,
            envelope::sources(&envelope),
            network_address::net_id(&to),
            cs_message,
            sendSn,
            false,
            ctx);
        };

        event::emit(CallMessageSent{from:network_address::net_id(&from),to:network_address::net_id(&to),sn:sequence_no});      
    }

    fun connection_send_message(self:&mut Storage,fee:&mut Coin<SUI>,sources:vector<String>, net_to:String, cs_message:CSMessage,sn:u128,is_response:bool,ctx: &mut TxContext){
        let mut protocols=sources;
        if(vector::is_empty(&sources)){
            let connection= xcall_state::get_connection(self,net_to);
            vector::push_back(&mut protocols,connection);
           
        };
        
            let cs_message_bytes=cs_message::encode(&cs_message);
            let mut i=0;
            while(i < vector::length(&protocols)){
                let protocol=*vector::borrow(&protocols,i);
                connections::send_message(
                    xcall_state::get_connection_states_mut(self),
                    protocol,
                    fee,
                    net_to,
                    sn,
                    cs_message_bytes,
                    is_response,
                    ctx);
                i=i+1;
            };



    }

    fun get_next_sequence(self:&mut Storage):u128 {
        let sn=xcall_state::get_next_sequence(self);
        sn
    }


    fun get_next_req_id(self:&mut Storage):u128 {
        let req_id=xcall_state::get_next_request_id(self);
        req_id
    }

    entry fun set_admin(addr:address,ctx: &mut TxContext){}


    entry fun set_protocol_fee(self:&mut Storage,admin:&AdminCap,fee:u128){
        xcall_state::set_protocol_fee(self,fee);
    }

    entry fun set_protocol_fee_handler(self:&mut Storage,admin:&AdminCap,fee_handler:address){
        xcall_state::set_protocol_fee_handler(self,fee_handler);
    }

    entry public fun send_call(self:&mut Storage,fee: &mut Coin<SUI>,idCap:&IDCap,to:String,envelope_bytes:vector<u8>,ctx: &mut TxContext){
        let envelope=envelope::decode(&envelope_bytes);
        let to = network_address::from_string(to);
        let from= network_address::create(string::utf8(NID),string::utf8(object::id_to_bytes(&object::id(idCap))));

        send_call_inner(self,fee,from,to,envelope,ctx);
    }

    public fun handle_message(self:&mut Storage,
    cap:&ConnCap, 
    from:String, 
    msg:vector<u8>,
    ctx: &mut TxContext){
        assert!(from != string::utf8(NID),EInvalidNID);
        let cs_msg = cs_message::decode(&msg);
        let msg_type = cs_message::msg_type(&cs_msg);
        let payload = cs_message::payload(&cs_msg);

        if (msg_type == cs_message::request_code()) {
            handle_request(self,cap,from, payload, ctx);
        } else if (msg_type == cs_message::result_code()) {
           
            handle_result(self, payload, ctx);
        } else {
        }
    }

    fun handle_request(self:&mut Storage,cap:&ConnCap,from:String,payload:vector<u8>, ctx: &mut TxContext){
        let req = message_request::decode(&payload);
        let from_nid = message_request::from_nid(&req);
        let data = message_request::data(&req);
        let from = message_request::from(&req);
        let sn = message_request::sn(&req);
        let msg_type = message_request::msg_type(&req);

        assert!(from_nid == string::utf8(NID),EInvalidNID);

        let source = cap.package_id();
        let to = message_request::to(&req);
        let protocols = message_request::protocols(&req);

        let source_valid = is_valid_source(self, from_nid, source, message_request::protocols(&req));

        assert!(source_valid, EInvalidSource);

        if(vector::length(&protocols) > 1){
            let key = hash::keccak256(&payload);
            xcall_state::save_pending_requests(self, key, source);
            if(xcall_state::check_pending_requests(self, key, protocols)) return;

            xcall_state::remove_pending_requests(self, key, protocols);
        };

        let req_id = get_next_req_id(self);
        let proxy_request = message_request::create(from, to, sn, msg_type, data, protocols);
        self.add_proxy_request(req_id, proxy_request);
        event::emit(CallMessage{from, to, sn, req_id, data});
    }


    fun handle_result(self:&mut Storage,payload:vector<u8>, ctx: &mut TxContext){
        let cs_message_result = message_result::decode(&payload);
        let sequence_no = message_result::sequence_no(&cs_message_result);
        let code = message_result::response_code(&cs_message_result);
        let message = message_result::message(&cs_message_result);

        assert!(xcall_state::has_rollback(self, sequence_no), ENoRollback);
        let rollback = xcall_state::get_rollback(self, sequence_no);

        let sources = rollback_data::sources(&rollback);
        let to = rollback_data::to(&rollback);

        let source = address::to_string(tx_context::sender(ctx));

        let source_valid = is_valid_source(self, to, source, sources);

        assert!(source_valid, EInvalidSource);

        if(vector::length(&sources) > 1){
            let key = hash::keccak256(&payload);
            xcall_state::save_pending_responses(self, key, source);
            let i = 0;
            if(xcall_state::check_pending_responses(self, key, sources)) return;

            xcall_state::remove_pending_responses(self, key, sources);
        };
        event::emit(ResponseMessage { sn: sequence_no, response_code: code });

        if (code == message_result::success()) {
        if (vector::length(&message) > 0) {
            let msg = message_request::decode(&message);
            handle_reply(self,&rollback, &msg, ctx);
        };
        xcall_state::set_successful_responses(self, sequence_no);
        cleanup_call_request(self, sequence_no);
    } else {
        let mut_rollback = xcall_state::get_mut_rollback(self, sequence_no);
        rollback_data::enable_rollback(mut_rollback);
        xcall_state::add_rollback(self, sequence_no, *mut_rollback);
        event::emit(RollbackMessage{sn:sequence_no})
    };
    }

    fun cleanup_call_request(self:&mut Storage,sequence_no:u128){
        xcall_state::remove_rollback(self, sequence_no);
    }

    fun handle_reply(self:&mut Storage, rollback:&RollbackData, reply: &CSMessageRequest, ctx: &mut TxContext){
        assert!(rollback_data::to(rollback) == message_request::from_nid(reply), EInvalidReply);

        let req_id = get_next_req_id(self);

        let from = message_request::from(reply);
        let to = message_request::to(reply);
        let data = message_request::data(reply);
        let protocols = message_request::protocols(reply);
        let sn = message_request::sn(reply);
        event::emit(CallMessage{from,to, sn, req_id,data});

    }


    public fun execute_call(self:&mut Storage,cap:&IDCap,request_id:u128,data:vector<u8>,ctx: &mut TxContext):ExecuteTicket{

        let proxy_request = xcall_state::get_proxy_request(self, request_id);
        let from = message_request::from(proxy_request);
        let to = message_request::to(proxy_request);
        let sn = message_request::sn(proxy_request);
        let msg_type = message_request::msg_type(proxy_request);
        let msg_data=message_request::data(proxy_request);
        let data_hash = hash::keccak256(&msg_data);
        let protocols = message_request::protocols(proxy_request);

        assert!(data_hash == data, EDataMismatch);
        if(msg_type==CALL_MESSAGE_ROLLBACK_TYPE){
            xcall_state::set_reply_state(self, *proxy_request);
        };
        let ticket=execute_ticket::new(
            xcall_state::get_id_cap_id(cap),
            request_id,
            from,
            msg_data,
            
        );   
        ticket
    }

    public fun execute_call_result(self:&mut Storage,ticket:ExecuteTicket,success:bool,fee:&mut Coin<SUI>,ctx:&mut TxContext){
        let request_id=execute_ticket::request_id(&ticket);
        let proxy_request = xcall_state::get_proxy_request(self, request_id);
        let msg_type = message_request::msg_type(proxy_request);
        let sn = message_request::sn(proxy_request);
        let from=message_request::from(proxy_request);
        let net_to=network_address::net_id(&from);
        let protocols=message_request::protocols(proxy_request);

        if(msg_type==PERSISTENT_MESSAGE_TYPE && !success){
            assert!(1==2,0x01);
        };
        cleanup_call_request(self, sn);
        xcall_state::remove_reply_state(self);
        let mut message = vector::empty<u8>();
        let code= if(success){1}else{0};
        let cs_message_result = if(msg_type==CALL_MESSAGE_ROLLBACK_TYPE){
            let callReply = xcall_state::get_call_reply(self);
            if(vector::length(&callReply) > 0 && code == 0){
                message = callReply;
                xcall_state::remove_call_reply(self);
            };
            message_result::create(sn, code, message)

        }else {
            message_result::create(sn, code, message)
        };
        execute_ticket::consume(ticket);
        if(msg_type==CALL_MESSAGE_ROLLBACK_TYPE){
          let cs_message=cs_message::from_message_result(cs_message_result);
           connection_send_message(self,fee,protocols,net_to,cs_message,sn,true,ctx);

        };
       


    }

    public fun execute_rollback(self:&mut Storage,cap:&IDCap, sn:u128,ctx: &mut TxContext):RollbackTicket{
        assert!(xcall_state::has_rollback(self, sn), ENoRollback);
        let rollback = xcall_state::get_rollback(self, sn);
        let rollback_data= rollback_data::rollback(&rollback);
        assert!(!rollback_data::enabled(&rollback), ERollbackNotEnabled);
        let ticket=rollback_ticket::new(sn,rollback_data,xcall_state::get_id_cap_id(cap));
        ticket
        

    }

    public fun execute_rollback_result(self:&mut Storage,ticket:RollbackTicket,success:bool){
        let sn= rollback_ticket::sn(&ticket);
        if(success){
         cleanup_call_request(self, sn);
         event::emit(RollbackExecuted{sn})
        };
        rollback_ticket::consume(ticket);


    }

    fun is_valid_source(self:&mut Storage,nid:String,source:String,protocols:vector<String>):bool{

        if(vector::contains(&protocols,&source)){
            return true
        };
        let connection = xcall_state::get_connection(self, nid);
        (connection == source)
    }

    fun isReply(self:&mut Storage,net_id:String, sources: vector<String>):bool{
        let reply_state = xcall_state::get_reply_state(self);
        return message_request::from_nid(&reply_state) == net_id && 
            message_request::protocols(&reply_state) == sources

    }

    entry fun verify_success(self:&mut Storage,sn:u128,ctx: &mut TxContext){
        xcall_state::get_successful_responses(self, sn);
    }

    entry fun migrate(self: &mut Storage, a: &AdminCap) {
        assert!(xcall_state::get_admin(self) == object::id(a), ENotAdmin);
        assert!(xcall_state::get_version(self) < CURRENT_VERSION, ENotUpgrade);
        xcall_state::set_version(self, CURRENT_VERSION);
       
    }

    #[test_only] use sui::test_scenario::{Self,Scenario};
    #[test_only]
    public fun init_xcall_state(admin:address,mut scenario:Scenario):Scenario{
     init(scenario.ctx());
     scenario.next_tx(admin);
     scenario
    }

    
}
