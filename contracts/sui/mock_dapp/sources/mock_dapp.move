module mock_dapp::mock_dapp {
    use xcall::main::{Self as xcall};
    use xcall::xcall_state::{Storage as XCallState};
    use xcall::network_address::{Self,NetworkAddress};
    use mock_dapp::dapp_state::{Self,DappState};
    use xcall::execute_ticket::{Self};
    use xcall::envelope::{Self};
    use sui::coin::{Self, Coin};
    use sui::sui::SUI;
     use std::string::{Self, String};


public struct REGISTER_WITNESS has store,drop {}
public struct WitnessCarrier has key { id: UID, witness: REGISTER_WITNESS }


    fun init(ctx: &mut TxContext) {


         transfer::transfer(
            WitnessCarrier { id: object::new(ctx), witness:REGISTER_WITNESS{} },
            ctx.sender()
        );
       
    }

    entry public fun register_xcall(xcall:&XCallState,carrier:WitnessCarrier,ctx:&mut TxContext){
        let w= get_witness(carrier);
        let cap= xcall::register_dapp(xcall,w,ctx);
        let state=dapp_state::new(cap,ctx);
        dapp_state::share(state);

    }

     fun get_witness(carrier: WitnessCarrier): REGISTER_WITNESS {
        let WitnessCarrier { id, witness } = carrier;
        id.delete();
        witness
    }

    entry public fun execute_call(state:&mut DappState,xcall:&mut XCallState,fee: &mut Coin<SUI>,request_id:u128,data:vector<u8>,ctx:&mut TxContext){
        let ticket= xcall::execute_call(xcall,dapp_state::get_xcall_cap(state),request_id,data,ctx);
        let msg= execute_ticket::message(&ticket);
        let from=execute_ticket::from(&ticket);
        if(msg==b"rollback"){
             xcall::execute_call_result(xcall,ticket,false);

        }else if(msg==b"reply-response"){
             send_message(state,xcall,fee,from,vector::empty<u8>(),ctx);
             xcall::execute_call_result(xcall,ticket,true);

        }else {
          xcall::execute_call_result(xcall,ticket,true);
        };

       


    }

    entry public fun execute_rollback(state:&mut DappState,xcall:&mut XCallState, sn:u128,ctx: &mut TxContext){
         let ticket= xcall::execute_rollback(xcall,dapp_state::get_xcall_cap(state),sn,ctx);
         xcall::execute_rollback_result(xcall,ticket,true);

    }

    entry public fun add_connection(state:&mut DappState,net_id:String,source:String,destination:String){
        dapp_state::add_connection(state,net_id,source,destination);
    }

    fun send_message(state:&mut DappState,xcall:&mut XCallState,fee: &mut Coin<SUI>,to:NetworkAddress,data:vector<u8>,ctx: &mut TxContext){
        let connection= dapp_state::get_connection(state,network_address::net_id(&to));
        let sources=dapp_state::get_connection_source(&connection);
        let destinations=dapp_state::get_connection_dest(&connection);
        let envelope=envelope::wrap_call_message(data,sources,destinations);
        let envelope_bytes=envelope::encode(&envelope);
        xcall::send_call(xcall,fee,dapp_state::get_xcall_cap(state),network_address::to_string(&to),envelope_bytes,ctx);
    }

    

    


}