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
   
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::envelope::{Self,XCallEnvelope};
    use xcall::connections::{Self,register};
    use xcall::message_request::{Self, CSMessageRequest};
    use xcall::call_message_rollback::{Self,CallMessageWithRollback};
    use xcall::message_result::{Self};
    use xcall::cs_message::{Self};
    use xcall::rollback_data::{Self,RollbackData};
    use xcall::xcall_state::{Self,Storage,AdminCap,IDCap};
    use xcall::execute_ticket::{Self,ExecuteTicket};
    use sui::bag::{Bag, Self};
    use sui::table::{Table,Self};
    use sui::package::{Self,Publisher};
  
    use sui::vec_map::{Self, VecMap};
    use sui::versioned::{Self, Versioned};
    use sui::sui::SUI;
     use sui::coin::{Self, Coin};
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

    const CS_REQUEST: u8 =0;
    const CS_RESULT: u8 =1;

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
       // transfer::transfer(admin, tx_context::sender(ctx));
       xcall_state::transfer_admin_cap(admin,ctx);
    }

    public fun register_dapp<T: drop>(self:&Storage,
        witness: T,
        ctx: &mut TxContext
    ):IDCap {
        // assert!(sui_types::is_one_time_witness(&witness), ENotOneTimeWitness);

        xcall_state::create_id_cap(self,ctx)

       
    }

    public fun register_connection(self:&mut Storage,net_id:String,package_id:String){
        xcall_state::set_connection(self,net_id,package_id);
        register(xcall_state::get_connection_states(self),package_id);
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
                network_address::net_id(&to),
                envelope::sources(&envelope),
                call_message_rollback::rollback(&msg),
                false
            );

            xcall_state::add_rollback(self,sequence_no,rollback);

            need_response = true;
            data = call_message_rollback::data(&msg);
        }
        else{
            abort EInvalidMsgType
        };


        let dst_account = network_address::addr(&to);

        let cs_request= message_request::create(
            from,
            dst_account,
            sequence_no,
            envelope::msg_type(&envelope),
            data,
            envelope::destinations(&envelope));

        let msg = message_request::encode(&cs_request);

        assert!(vector::length(&msg) <= MAX_DATA_SIZE, EInvalidReply);

        if(isReply(self,network_address::net_id(&to),envelope::sources(&envelope))){
            xcall_state::remove_reply_state(self);
            xcall_state::set_call_reply(self,msg);
        } else{
            let sendSn = if (need_response) {sequence_no} else {0};


            send_message(self,
    fee,envelope::sources(&envelope),network_address::net_id(&to),msg_type,msg,sendSn,ctx);
        };

        event::emit(CallMessageSent{from:network_address::net_id(&from),to:network_address::net_id(&to),sn:sequence_no});        
    }

    fun send_message(self:&mut Storage,fee:&mut Coin<SUI>,sources:vector<String>, net_to:String, msg_type:u8, data:vector<u8>,sn:u128,ctx: &mut TxContext){
        let mut sources=sources;
        if(vector::is_empty(&sources)){
            let connection= xcall_state::get_connection(self,net_to);
            vector::push_back(&mut sources,connection);
            // let required_fee = xcall_state::get_protocol_fee(self);
            // let connection_coin = coin::split(fee, required_fee, ctx);
            // connections::send_message(package_id,connection,connection_coin,required_fee,net_to,msg_type,sn,data,ctx);
        } else{
            let mut i=0;
            while(i < vector::length(&sources)){
                // let required_fee = xcall_state::get_protocol_fee(self);
                // let connection_coin = coin::split(fee, required_fee, ctx);
                // connections::send_message(sources[i],required_fee,net_to,msg_type,sn,data,ctx);
                i=i+1
            }
        }
    }

    fun get_next_sequence(self:&mut Storage):u128 {
        let sn=xcall_state::get_next_sequence(self);
        sn
    }

    fun get_fee(self:&mut Storage):u128 {
        // xcall_state::get_protocol_fee(self)
        0
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

    entry fun send_call_message(self:&mut Storage,fee: &mut Coin<SUI>,idCap:&IDCap,to:String,data:vector<u8>,rollback:vector<u8>, sources:vector<String>, destinations:vector<String>,ctx: &mut TxContext){
        let envelope;
        if(vector::length(&rollback) > 0){
            envelope = envelope::wrap_call_message(data, sources, destinations);
        } else {
            envelope = envelope::wrap_call_message_rollback(data, rollback, sources, destinations);
        };
        send_call(self,fee,idCap,to,envelope::encode(&envelope),ctx);
    }

    entry fun send_call(self:&mut Storage,fee: &mut Coin<SUI>,idCap:&IDCap,to:String,envelope_bytes:vector<u8>,ctx: &mut TxContext){
        let envelope=envelope::decode(&envelope_bytes);
        let to = network_address::from_string(to);
        let from= network_address::create(string::utf8(NID),string::utf8(object::id_to_bytes(&object::id(idCap))));

        send_call_inner(self,fee,from,to,envelope,ctx);
    }

    entry fun handle_message(self:&mut Storage, from:String, msg:vector<u8>,ctx: &mut TxContext){
        assert!(from != string::utf8(NID),EInvalidNID);
        let cs_msg = cs_message::decode(&msg);
        let msg_type = cs_message::msg_type(&cs_msg);
        let payload = cs_message::payload(&cs_msg);

        if (msg_type == CS_REQUEST) {
            handle_request(self,from, payload, ctx);
        } else if (msg_type == CS_RESULT) {
            let cs_message_result = message_result::decode(&payload);
            handle_result(self, cs_message_result, ctx);
        } else {
        }
    }

    fun handle_request(self:&mut Storage,from:String,payload:vector<u8>, ctx: &mut TxContext){
        let req = message_request::decode(&payload);
        let from_nid = message_request::from_nid(&req);
        let data = message_request::data(&req);
        let from = message_request::from(&req);
        let sn = message_request::sn(&req);
        let msg_type = message_request::msg_type(&req);

        assert!(from_nid == string::utf8(NID),EInvalidNID);

        let source = address::to_string(tx_context::sender(ctx));
        let to = message_request::to(&req);
        let protocols = message_request::protocols(&req);

        let source_valid = is_valid_source(self, from_nid, source, message_request::protocols(&req));

        assert!(source_valid, EInvalidSource);

        if(vector::length(&protocols) > 1){
            let key = b"";
            xcall_state::save_pending_requests(self, key, source);
            let i = 0;
            if(xcall_state::check_pending_requests(self, key, protocols)) return;

            xcall_state::remove_pending_requests(self, key, protocols);
        };

        let req_id = get_next_req_id(self);

        let proxy_request = message_request::create(from, to, sn, msg_type, data, protocols);

        xcall_state::add_proxy_request(self, req_id, proxy_request);
        event::emit(CallMessage{from, to, sn, req_id, data});
    }


    fun handle_result(self:&mut Storage,cs_message_result: message_result::CSMessageResponse, ctx: &mut TxContext){
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
            let key = b"";
            xcall_state::save_pending_responses(self, key, source);
            let i = 0;
            if(xcall_state::check_pending_responses(self, key, sources)) return;

            xcall_state::remove_pending_responses(self, key, sources);
        };
        event::emit(ResponseMessage { sn: sequence_no, response_code: code });

        if (code == 0) {
        cleanup_call_request(self, sequence_no);

        if (vector::length(&message) > 0) {
            let msg = message_request::decode(&message);
            // handle_reply(self,&rollback, &msg, ctx);
        };

        xcall_state::set_successful_responses(self, sequence_no);
    } 
    else {

        assert!(xcall_state::has_rollback(self, sequence_no), ENoRollback);
        let mut_rollback = xcall_state::get_mut_rollback(self, sequence_no);

        rollback_data::enable_rollback(mut_rollback);
        xcall_state::add_rollback(self, sequence_no, *mut_rollback);
        event::emit(RollbackMessage{sn:sequence_no})
    }
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
        // let sources = message_request::sources(reply);

        event::emit(CallMessage{from,to, sn, req_id,data});






        // let data = message_request::encode(reply);
        // execute_call(self, reqId, data, ctx);

    }

    entry fun handle_error(self:&mut Storage, sn:u128,ctx: &mut TxContext){

        let cs_message_result = message_result::create(sn, 1, vector::empty<u8>());
        // handle_result(self, sn, ctx);
        handle_result(self, cs_message_result, ctx);
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

    public fun execute_call_result(self:&mut Storage,ticket:ExecuteTicket,success:bool){
        let request_id=execute_ticket::request_id(&ticket);
        let proxy_request = xcall_state::get_proxy_request(self, request_id);
        let msg_type = message_request::msg_type(proxy_request);
        let sn = message_request::sn(proxy_request);

        if(msg_type==PERSISTENT_MESSAGE_TYPE && !success){
            assert!(1==2,0x01);
        };
        cleanup_call_request(self, sn);
        xcall_state::remove_reply_state(self);
        let mut message = vector::empty<u8>();
        let code= if(success){1}else{0};
        let cs_message = if(msg_type==CALL_MESSAGE_ROLLBACK_TYPE){
            let callReply = xcall_state::get_call_reply(self);
            if(vector::length(&callReply)>0 && code == 0){
                message = callReply;
                xcall_state::remove_call_reply(self);
            };
            message_result::create(sn, code, message)

        }else {
            message_result::create(sn, code, message)
        };
        execute_ticket::consume(ticket);

       // send message flow
       


    }

    fun try_execute_call(self:&mut Storage,req_id:u128, dapp: String, from:NetworkAddress, data: vector<u8>, protocols:vector<String>,ctx: &mut TxContext):u8{
        0
    }

    fun execute_message(self:&mut Storage,to: String, from:NetworkAddress, data: vector<u8>, protocols:vector<String>,ctx: &mut TxContext){

    }

    entry fun execute_rollback(self:&mut Storage,cap:&IDCap, sn:u128,ctx: &mut TxContext){
        assert!(xcall_state::has_rollback(self, sn), ENoRollback);
        let rollback = xcall_state::get_rollback(self, sn);
        assert!(!rollback_data::enabled(&rollback), ERollbackNotEnabled);
        

        cleanup_call_request(self, sn);

       // execute_message(self, address::to_string(rollback_data::from(&rollback)), network_address::from_string(string::utf8(b"")), rollback_data::rollback(&rollback), rollback_data::sources(&rollback), ctx);

        event::emit(RollbackExecuted{sn})
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
}
