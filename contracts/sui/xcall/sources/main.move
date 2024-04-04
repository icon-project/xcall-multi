#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::main {

    // Part 1: Imports
    use sui::object::{Self, UID,ID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::linked_table::{Self, LinkedTable};
    use sui::types as sui_types;
    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option};
   
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::envelope::{Self,XCallEnvelope};
    use xcall::connections::{Self,register};
    use xcall::message_request::{Self};
    use xcall::cs_message::{Self};
    use xcall::rollback_data::{Self,RollbackData};
    use xcall::xcall_state::{Self,Storage,AdminCap,IDCap};
    use sui::bag::{Bag, Self};
    use sui::table::{Table,Self};
    use sui::package::{Self,Publisher};
  
    use sui::vec_map::{Self, VecMap};
    use sui::versioned::{Self, Versioned};
    use sui::sui::SUI;
     use sui::coin::{Self, Coin};


    const ENotOneTimeWitness: u64 = 0;
    const ENotAdmin: u64 = 1;
    const ENotUpgrade: u64 = 2;
    const EWrongVersion: u64 = 3;

    const NID: vector<u8> = b"nid";

    const CURRENT_VERSION: u64 = 1;

    
    
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
        assert!(sui_types::is_one_time_witness(&witness), ENotOneTimeWitness);

        xcall_state::create_id_cap(self,ctx)

       
    }

    public fun register_connection(self:&mut Storage,net_id:String,package_id:String){
        xcall_state::set_connection(self,net_id,package_id);
        register(xcall_state::get_connection_states(self),package_id);
    }

    fun send_call_inner(self:&mut Storage,
    fee: Coin<SUI>,from:NetworkAddress,to:NetworkAddress,envelope:XCallEnvelope,ctx: &mut TxContext){
        /*
         let caller = info.sender.clone();
        let config = self.get_config(deps.as_ref().storage)?;
        let nid = config.network_id;
        self.validate_payload(deps.as_ref(), &caller, &envelope)?;

        let sequence_no = self.get_next_sn(deps.storage)?;

        let from = NetworkAddress::new(&nid, caller.as_ref());
         if envelope.message.rollback().is_some() {
            let rollback_data = envelope.message.rollback().unwrap();
            let request = Rollback::new(
                caller.clone(),
                to.clone(),
                envelope.sources.clone(),
                rollback_data,
                false,
            );

            self.store_call_request(deps.storage, sequence_no, &request)?;
        }
        let call_request = CSMessageRequest::new(
            from,
            to.account(),
            sequence_no,
            envelope.message.msg_type().clone(),
            envelope.message.data(),
            envelope.destinations,
        );
        let need_response = call_request.need_response();

        let event = event_xcall_message_sent(caller.to_string(), to.to_string(), sequence_no);
        // if contract is in reply state
        if envelope.message.rollback().is_none()
            && self.is_reply(deps.as_ref(), to.nid(), &envelope.sources)
        {
            self.save_call_reply(deps.storage, &call_request)?;
            let res = self.send_call_response(event, sequence_no);
            return Ok(res);
        }

        let mut confirmed_sources = envelope.sources;
        if confirmed_sources.is_empty() {
            let default = self.get_default_connection(deps.as_ref().storage, to.nid())?;
            confirmed_sources = vec![default.to_string()]
        }
        let message: CSMessage = call_request.into();
        let sn: i64 = if need_response { sequence_no as i64 } else { 0 };
        let mut total_spent = 0_u128;

        let submessages = confirmed_sources
            .iter()
            .map(|r| {
                return self
                    .query_connection_fee(deps.as_ref(), to.nid(), need_response, r)
                    .and_then(|fee| {
                        let fund = if fee > 0 {
                            total_spent = total_spent.checked_add(fee).unwrap();
                            coins(fee, config.denom.clone())
                        } else {
                            vec![]
                        };
                        let address = deps.api.addr_validate(r)?;

                        self.call_connection_send_message(&address, fund, to.nid(), sn, &message)
                    });
            })
            .collect::<Result<Vec<SubMsg>, ContractError>>()?;

        let total_paid = self.get_total_paid(deps.as_ref(), &info.funds)?;
        let fee_handler = self.fee_handler().load(deps.storage)?;
        let protocol_fee = self.get_protocol_fee(deps.as_ref().storage);
        let total_fee_required = protocol_fee + total_spent;

        if total_paid < total_fee_required {
            return Err(ContractError::InsufficientFunds);
        }
        let remaining = total_paid - total_spent;

        println!("{LOG_PREFIX} Sent Bank Message");
        let mut res = self
            .send_call_response(event, sequence_no)
            .add_submessages(submessages);

        if remaining > 0 {
            let msg = BankMsg::Send {
                to_address: fee_handler,
                amount: coins(remaining, config.denom),
            };
            res = res.add_message(msg);
        }

        Ok(res)
        
        
        
        */
        let sequence_no=get_next_sequence(self);
        let rollback=envelope::rollback(&envelope);
        if(option::is_some(&rollback)){
            let rollback_bytes=option::extract<vector<u8>>(&mut rollback);
            let rollback= rollback_data::create(copy from,to,envelope::sources(&envelope),rollback_bytes,false);
           // table::add(&mut self.rollbacks,sequence_no,rollback);
            xcall_state::add_rollback(self,sequence_no,rollback);

        };

        let cs_request= message_request::create(
            from,
            network_address::addr(&to),
            sequence_no,
            envelope::msg_type(&envelope),
            envelope::message(&envelope),
            envelope::sources(&envelope));
        let sources=envelope::sources(&envelope);
        if(vector::is_empty(&sources)){
            sources=vector::empty<String>();
            let connection= xcall_state::get_connection(self,network_address::net_id(&to));
            vector::push_back(&mut sources,connection);
        };
         transfer::public_transfer(fee, tx_context::sender(ctx));

    }

    fun get_next_sequence(self:&mut Storage):u128 {
        let sn=xcall_state::get_next_sequence(self);
        sn
    }

    entry fun set_admin(addr:address,ctx: &mut TxContext){}


    entry fun set_protocol_fee(self:&mut Storage,admin:&AdminCap,fee:u128){
        xcall_state::set_protocol_fee(self,fee);
    }

    entry fun set_protocol_fee_handler(self:&mut Storage,admin:&AdminCap,fee_handler:address){
        xcall_state::set_protocol_fee_handler(self,fee_handler);
    }

    entry fun send_call(self:&mut Storage,fee: Coin<SUI>,idCap:&IDCap,to:String,envelope_bytes:vector<u8>,ctx: &mut TxContext){
        let envelope=envelope::decode(envelope_bytes);
        let to = network_address::from_string(to);
        let from= network_address::create(string::utf8(NID),string::utf8(object::id_to_bytes(&object::id(idCap))));
        send_call_inner(self,fee,from,to,envelope,ctx)
    }

    entry fun handle_message(self:&mut Storage, from:String, msg:vector<u8>,ctx: &mut TxContext){}


    entry fun handle_error(self:&mut Storage, sn:u128,ctx: &mut TxContext){}

    entry fun execute_call(self:&mut Storage,request_id:u128,data:vector<u8>,ctx: &mut TxContext){}
    #[allow(unused_field)]
    entry fun execute_rollback(self:&mut Storage,sn:u128,ctx: &mut TxContext){}

    


    entry fun migrate(self: &mut Storage, a: &AdminCap) {
        assert!(xcall_state::get_admin(self) == object::id(a), ENotAdmin);
        assert!(xcall_state::get_version(self) < CURRENT_VERSION, ENotUpgrade);
        xcall_state::set_version(self, CURRENT_VERSION);
       
    }

    

    


}