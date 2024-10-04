#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::main {

    use sui::linked_table::{Self, LinkedTable};
    use sui::types as sui_types;
    use std::string::{Self, String};
    use sui::event;
    use sui::hash::{Self};
    use sui::coin::{Self,Coin};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    use xcall::xcall_utils as utils;

    use xcall::network_address::{Self,NetworkAddress};
    use xcall::envelope::{Self,XCallEnvelope};
    use xcall::connections::{Self,register};
    use xcall::message_request::{Self, CSMessageRequest};
    use xcall::call_message::{Self};
    use xcall::persistent_message::{Self};
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
    use sui::package::UpgradeCap;


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
    const EInvalidAccess:u64 =12;
    const EInvalidMsgCode:u64 =13;
    const EDataTooBig:u64 =14;
    const EInvalidConnectionId:u64 =15;

    const MAX_DATA_SIZE: u64 = 2048;
    const CURRENT_VERSION: u64 = 3;

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

    public struct CallExecuted has copy, drop{
        req_id:u128,
        code:u8,
        err_msg: String
    }

    public struct RollbackMessage has copy, drop{
        sn:u128,
        dapp: String,
        data:vector<u8>,
    }

    public struct RollbackExecuted has copy, drop{
        sn:u128
    }

    public struct ResponseMessage has copy, drop{
        sn:u128,
        response_code: u8
    }
    /***************/
   
    
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

    public fun register_dapp<T: drop>(self:&Storage,
    witness: T,
    ctx: &mut TxContext
    ):IDCap {
        self.create_id_cap(ctx)
    }

    entry public fun configure_nid(self:&mut Storage,owner:&UpgradeCap, net_id:String,ctx: &mut TxContext){
        if(!(net_id == b"".to_string())){
            xcall_state::set_net_id(self,net_id);
        }
    }

    entry public fun get_nid(self: &mut Storage): String{
        xcall_state::get_net_id(self)
    }


    entry public fun register_connection_admin(self:&mut Storage,admin:&AdminCap,connection_id:String,relayer:address,ctx: &mut TxContext){
        self.enforce_version(CURRENT_VERSION);
        let cap= xcall_state::new_conn_cap(self.get_id(),connection_id,ctx);
        xcall_state::transfer_conn_cap(cap,relayer,ctx);
        register(self.get_connection_states_mut(),connection_id,ctx);
    }

    entry public fun set_default_connection(self:&mut Storage,admin:&AdminCap,net_id:String,connection_id:String,ctx: &mut TxContext){
        self.enforce_version(CURRENT_VERSION);
        let connection_exists=bag::contains(self.get_connection_states(),connection_id);
        if(!connection_exists){
            abort EInvalidConnectionId
        };
        self.set_connection(net_id,connection_id);
    }

    public fun admin(self:&mut Storage):ID{
        xcall_state::get_admin(self)
    }

    public fun get_fee_handler(self:&mut Storage):address{
        xcall_state::get_protocol_fee_handler(self)
    }

    fun get_connection_fee(self:&Storage,connection_id:String,net_id:String, needs_response:bool):u64{
        connections::get_fee(
            xcall_state::get_connection_states(self),
            connection_id,
            net_id,
            needs_response
        )
    }

    entry public fun get_fee(self:&Storage, net_id:String, rollback:bool, sources:Option<vector<String>>):u64{
        let mut fee = xcall_state::get_protocol_fee(self);
        let mut sources= sources.get_with_default(vector::empty<String>());

        if (sources.is_empty()){
            let connection = xcall_state::get_connection(self,net_id);
            sources.push_back(connection);
        };

        if(isReply(self,net_id,sources) && !rollback){
            return 0
        };

        let mut i = 0;
        while(i < vector::length(&sources)){
            let source = vector::borrow(&sources, i);
            fee = fee + get_connection_fee(self,*source, net_id, rollback);
            i=i+1
        };

        fee
    }


    fun send_call_inner(self:&mut Storage,fee:Coin<SUI>,from:NetworkAddress,to:NetworkAddress,envelope:XCallEnvelope,ctx: &mut TxContext):Coin<SUI>{
        self.enforce_version(CURRENT_VERSION);
        let sequence_no=get_next_sequence(self);
        let msg_type=envelope::msg_type(&envelope);

        let mut need_response = false;
        let data;

        if(msg_type == call_message::msg_type() || msg_type == persistent_message::msg_type()){
            data = envelope::message(&envelope);
        }
        else if(msg_type == call_message_rollback::msg_type()){
            let msg = call_message_rollback::decode(&envelope::message(&envelope));
            let from_id = utils::id_from_hex_string(&network_address::addr(&from));
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

        let cs_request= message_request::create(
        from,
        to.addr(),
        sequence_no,
        envelope.msg_type(),
        data,
        envelope.destinations());

        let msg = message_request::encode(&cs_request);

        assert!(vector::length(&msg) <= MAX_DATA_SIZE, EDataTooBig);

        let fee = if(isReply(self,to.net_id(),envelope::sources(&envelope))){
            xcall_state::remove_reply_state(self);
            xcall_state::set_call_reply(self,msg);
            fee
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
            ctx)
        };

        event::emit(CallMessageSent{from:network_address::addr(&from),to:network_address::to_string(&to),sn:sequence_no});  
        fee    
    }

    fun connection_send_message(self:&mut Storage,mut fee:Coin<SUI>,sources:vector<String>, net_to:String, cs_message:CSMessage,sn:u128,is_response:bool,ctx: &mut TxContext):Coin<SUI>{
        let mut protocols=sources;
        if(vector::is_empty(&sources)){
            let connection= xcall_state::get_connection(self,net_to);
            vector::push_back(&mut protocols,connection);

        };

        let cs_message_bytes=cs_message::encode(&cs_message);
        let mut i=0;
        while(i < vector::length(&protocols)){
            let protocol=*vector::borrow(&protocols,i);

            let required_fee = if (is_response){
                0
            } else {
                get_connection_fee(self, protocol, net_to, sn > 0)
            };

            let paid= fee.split(required_fee,ctx);
            connections::send_message(
                xcall_state::get_connection_states_mut(self),
                protocol,
                paid,
                net_to,
                sn,
                cs_message_bytes,
                is_response,
                ctx);

            i=i+1;
        };

        fee
    }

    fun get_next_sequence(self:&mut Storage):u128 {
        let sn=xcall_state::get_next_sequence(self);
        sn
    }


    fun get_next_req_id(self:&mut Storage):u128 {
        let req_id=xcall_state::get_next_request_id(self);
        req_id
    }

    entry fun set_protocol_fee(self:&mut Storage,admin:&AdminCap,fee:u64){
        self.enforce_version(CURRENT_VERSION);
        xcall_state::set_protocol_fee(self,fee);
    }


    entry fun set_protocol_fee_handler(self:&mut Storage,admin:&AdminCap,fee_handler:address){
        self.enforce_version(CURRENT_VERSION);
        xcall_state::set_protocol_fee_handler(self,fee_handler);
    }

    public fun send_call(self:&mut Storage,fee: Coin<SUI>,idCap:&IDCap,to:String,envelope_bytes:vector<u8>,ctx: &mut TxContext){
        let envelope=envelope::decode(&envelope_bytes);
        let to = network_address::from_string(to);
        let from= network_address::create(xcall_state::get_net_id(self),utils::id_to_hex_string(&object::id(idCap)));

        let mut remaining = send_call_inner(self,fee,from,to,envelope,ctx);
        let protocol_fee= remaining.split(self.get_protocol_fee(),ctx);
        let fee_handler=self.get_protocol_fee_handler();
        transfer::public_transfer(protocol_fee,fee_handler);
        transfer::public_transfer(remaining, ctx.sender());
    }

    entry fun send_call_ua(self:&mut Storage,fee: Coin<SUI>,to:String,envelope_bytes:vector<u8>,ctx: &mut TxContext){
        let envelope=envelope::decode(&envelope_bytes);
        if (envelope.msg_type() == call_message_rollback::msg_type()){
                abort EInvalidMsgType
        };
        let to = network_address::from_string(to);
        let from= network_address::create(xcall_state::get_net_id(self),utils::address_to_hex_string(&ctx.sender()));

        let mut remaining = send_call_inner(self,fee,from,to,envelope,ctx);
        let protocol_fee= remaining.split(self.get_protocol_fee(),ctx);
        let fee_handler=self.get_protocol_fee_handler();
        transfer::public_transfer(protocol_fee,fee_handler);
        transfer::public_transfer(remaining, ctx.sender());
    }


   

    public fun handle_message(self:&mut Storage,
    cap:&ConnCap, 
    from:String, 
    msg:vector<u8>,
    ctx: &mut TxContext){
        self.enforce_version(CURRENT_VERSION);
        assert!(from != xcall_state::get_net_id(self),EInvalidNID);
        let cs_msg = cs_message::decode(&msg);
        let msg_type = cs_message::msg_type(&cs_msg);
        let payload = cs_message::payload(&cs_msg);

        if (msg_type == cs_message::request_code()) {
            handle_request(self,cap,from, payload, ctx);
        } else if (msg_type == cs_message::result_code()) {
            handle_result(self, cap, payload, ctx);
        } else {
            abort EInvalidMsgCode
        }
    }

    fun handle_request(self:&mut Storage,cap:&ConnCap,net_from:String,payload:vector<u8>, ctx: &mut TxContext){
        let req = message_request::decode(&payload);
        let from_nid = message_request::from_nid(&req);
        let data = message_request::data(&req);
        let from = message_request::from(&req);
        let sn = message_request::sn(&req);
        let msg_type = message_request::msg_type(&req);

        assert!(from_nid == net_from,EInvalidNID);

        let source = cap.connection_id();
        let to = message_request::to(&req);
        let protocols = message_request::protocols(&req);

        let source_valid = is_valid_source(self, from_nid, source, message_request::protocols(&req));

        assert!(source_valid, EInvalidSource);

        if(vector::length(&protocols) > 1){
            let key = hash::keccak256(&payload);
            xcall_state::save_pending_requests(self, key, source);
            if(!xcall_state::check_pending_requests(self, key, protocols)) return;
            xcall_state::remove_pending_requests(self, key, protocols);
        };
        let data_hash = hash::keccak256(&data);
        let req_id = get_next_req_id(self);
        let proxy_request = message_request::create(from, to, sn, msg_type, data_hash, protocols);
        self.add_proxy_request(req_id, proxy_request);
        event::emit(CallMessage{from, to, sn, req_id, data});
    }


    fun handle_result(self:&mut Storage,cap:&ConnCap,payload:vector<u8>, ctx: &mut TxContext){
        let cs_message_result = message_result::decode(&payload);
        let sequence_no = message_result::sequence_no(&cs_message_result);
        let code = message_result::response_code(&cs_message_result);
        let message = message_result::message(&cs_message_result);

        assert!(xcall_state::has_rollback(self, sequence_no), ENoRollback);

        let rollback = xcall_state::get_rollback(self, sequence_no);
        let sources = rollback_data::sources(&rollback);
        let to = rollback_data::to(&rollback);
        let source = cap.connection_id();
        let source_valid = is_valid_source(self, to, source, sources);

        assert!(source_valid, EInvalidSource);

        if(vector::length(&sources) > 1){
        let key = hash::keccak256(&payload);
            xcall_state::save_pending_responses(self, key, source);

            if(!xcall_state::check_pending_responses(self, key, sources)) return;

            xcall_state::remove_pending_responses(self, key, sources);
        };
        event::emit(ResponseMessage { sn: sequence_no, response_code: code });

        if (code == message_result::success()) {
            xcall_state::remove_rollback(self, sequence_no);

            if (vector::length(&message) > 0) {
                let msg = message_request::decode(&message);
                handle_reply(self,&rollback, &msg, ctx);
            };

            xcall_state::set_successful_responses(self, sequence_no);
        } else {
            let mut_rollback = xcall_state::get_mut_rollback(self, sequence_no);
            rollback_data::enable_rollback(mut_rollback);
            event::emit(RollbackMessage{sn:sequence_no, dapp: utils::id_to_hex_string(&rollback_data::from(&rollback)), data: rollback.rollback() });
        };
    }

    fun handle_reply(self:&mut Storage, rollback:&RollbackData, reply: &CSMessageRequest, ctx: &mut TxContext){

        assert!(rollback_data::to(rollback) == message_request::from_nid(reply), EInvalidReply);

        let from = reply.from();
        let to = reply.to();
        let data = reply.data();
        let protocols = rollback.sources();
        let sn = reply.sn();
        let msg_type = reply.msg_type();

        let req_id = get_next_req_id(self);
        let proxy_request = message_request::create(from, to, sn, msg_type, hash::keccak256(&data), protocols);
        self.add_proxy_request(req_id, proxy_request);
        event::emit(CallMessage{from,to, sn, req_id,data});

    }


    public fun execute_call(self:&mut Storage,cap:&IDCap,request_id:u128,data:vector<u8>,ctx: &mut TxContext):ExecuteTicket{
        self.enforce_version(CURRENT_VERSION);

        let proxy_request = xcall_state::get_proxy_request(self, request_id);
        let from = message_request::from(proxy_request);
        let to = message_request::to(proxy_request);
        let sn = message_request::sn(proxy_request);
        let msg_type = message_request::msg_type(proxy_request);
        let msg_data_hash=message_request::data(proxy_request);
        let protocols = message_request::protocols(proxy_request);
        let data_hash = hash::keccak256(&data);

        assert!(msg_data_hash == data_hash, EDataMismatch);
        std::debug::print(&to);
        std::debug::print(&utils::id_to_hex_string(&xcall_state::get_id_cap_id(cap)));
        assert!(to==utils::id_to_hex_string(&xcall_state::get_id_cap_id(cap)),EInvalidAccess);

        if(msg_type==call_message_rollback::msg_type()){
            xcall_state::set_reply_state(self, *proxy_request);
        };
        let ticket=execute_ticket::new(
            xcall_state::get_id_cap_id(cap),
            request_id,
            from,
            protocols,
            data,
        );   
        ticket
    }
    /**
    - `self`: A mutable reference to the `Storage` object.
    - `ticket`: An `ExecuteTicket` object containing details of the executed call.
    - `success`: A boolean indicating whether the call was successful.
    - `fee`: A mutable `Coin<SUI>` object representing the fee which can be zero since reply is already paid on source .
    - `ctx`: A reference to the transaction context (`TxContext`).
    **/
    public fun execute_call_result(self:&mut Storage,ticket:ExecuteTicket,success:bool,mut fee:Coin<SUI>,ctx:&mut TxContext){
        self.enforce_version(CURRENT_VERSION);
        let request_id=ticket.request_id();
        let proxy_request = xcall_state::get_proxy_request(self, request_id);
        let msg_type = message_request::msg_type(proxy_request);
        let sn = message_request::sn(proxy_request);
        let from=message_request::from(proxy_request);
        let net_to=network_address::net_id(&from);
        let protocols=message_request::protocols(proxy_request);

        if(msg_type==persistent_message::msg_type() && !success){
            abort 0x01
        };

        xcall_state::remove_proxy_request(self, request_id);
        xcall_state::remove_reply_state(self);

        let mut message = vector::empty<u8>();
        let code= if(success){1}else{0};
        let err_msg = if(success){string::utf8(b"success")}else{string::utf8(b"unknown error")};
        let cs_message_result = if(msg_type==call_message_rollback::msg_type()){
            let callReply = xcall_state::get_call_reply(self);
            if(vector::length(&callReply) > 0 && code == 1){
                message = callReply;
                xcall_state::remove_call_reply(self);
            };
            message_result::create(sn, code, message)

        }else {
            message_result::create(sn, code, message)
        };
        execute_ticket::consume(ticket);
        if(msg_type==call_message_rollback::msg_type()){
            let cs_message=cs_message::from_message_result(cs_message_result);
            fee = connection_send_message(self,fee,protocols,net_to,cs_message,sn,true,ctx);
        };

        transfer::public_transfer(fee, ctx.sender());
        event::emit(CallExecuted{req_id:request_id, code: code, err_msg: err_msg});

    }

    public fun execute_rollback(self:&mut Storage,cap:&IDCap, sn:u128,ctx: &mut TxContext):RollbackTicket{
        self.enforce_version(CURRENT_VERSION);
        assert!(xcall_state::has_rollback(self, sn), ENoRollback);
        let rollback = xcall_state::get_rollback(self, sn);
        let rollback_data= rollback_data::rollback(&rollback);
        assert!(rollback_data::from(&rollback) == object::id(cap), EInvalidAccess);
        assert!(rollback_data::enabled(&rollback), ERollbackNotEnabled);
        let ticket=rollback_ticket::new(sn,rollback_data,xcall_state::get_id_cap_id(cap));
        ticket
    }

    public fun execute_rollback_result(self:&mut Storage,ticket:RollbackTicket,success:bool){
        self.enforce_version(CURRENT_VERSION);
        let sn= rollback_ticket::sn(&ticket);
        if(success){
        xcall_state::remove_rollback(self, sn);
         event::emit(RollbackExecuted{sn})
        };
        rollback_ticket::consume(ticket);
    }

    fun is_valid_source(self:&Storage,nid:String,source:String,protocols:vector<String>):bool{
        if(vector::contains(&protocols,&source)){
            return true
        };
        if (protocols.is_empty()){
        let connection = xcall_state::get_connection(self, nid);
            return (connection == source)
        };
        return false
       
    }

    fun isReply(self:&Storage,net_id:String, sources: vector<String>):bool{
        let reply_state = xcall_state::get_reply_state(self);
        return message_request::from_nid(&reply_state) == net_id && 
            message_request::protocols(&reply_state) == sources

    }

    entry fun verify_success(self:&mut Storage,sn:u128,ctx: &mut TxContext){
        self.enforce_version(CURRENT_VERSION);
        xcall_state::get_successful_responses(self, sn);
    }

    entry fun migrate(self: &mut Storage, owner:&UpgradeCap) {
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

    #[test_only]
    public fun configure_nid_test(self:&mut Storage,owner:&AdminCap, net_id:String,ctx: &mut TxContext){
        if(!(net_id == b"".to_string())){
            xcall_state::set_net_id(self,net_id);
        }
    }

    
}
