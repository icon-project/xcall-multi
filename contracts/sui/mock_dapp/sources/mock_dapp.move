module mock_dapp::mock_dapp {
    use xcall::main::{Self as xcall};
    use xcall::xcall_utils;
    use xcall::xcall_state::{Self, Storage as XCallState};
    use xcall::network_address::{Self};
    use mock_dapp::dapp_state::{Self,DappState, ExecuteParams, create_execute_params, get_xcall_id};
    use xcall::execute_ticket::{Self};
    use xcall::envelope::{Self};
    use sui::coin::{Self, Coin};
    use sui::sui::SUI;
     use std::string::{Self, String};
    use sui::balance;

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
        let xcall_id = xcall_state::get_id_cap_xcall(&cap);
        let state=dapp_state::new(cap,xcall_id,ctx);


        dapp_state::share(state);

    }

     fun get_witness(carrier: WitnessCarrier): REGISTER_WITNESS {
        let WitnessCarrier { id, witness } = carrier;
        id.delete();
        witness
    }

    entry fun get_execute_params(config: &DappState, _msg:vector<u8>): ExecuteParams{
        let type_args:vector<String> = vector::empty();

        let mut result:vector<String> = vector::empty();
        result.push_back(xcall_utils::id_to_hex_string(&dapp_state::get_config_id(config)));
        result.push_back(xcall_utils::id_to_hex_string(&get_xcall_id(config)));
        result.push_back(b"coin".to_string());
        result.push_back(b"request_id".to_string());
        result.push_back(b"data".to_string());       
        create_execute_params(type_args, result)
    }

    entry fun get_rollback_params(config: &DappState, _msg:vector<u8>): ExecuteParams{
        let type_args:vector<String> = vector::empty();

        let mut result:vector<String> = vector::empty();
        result.push_back(xcall_utils::id_to_hex_string(&dapp_state::get_config_id(config)));
        result.push_back(xcall_utils::id_to_hex_string(&get_xcall_id(config)));
        result.push_back(b"sn".to_string());
        create_execute_params(type_args, result)
    }

    entry public fun execute_call(state:&mut DappState,xcall:&mut XCallState,mut fee: Coin<SUI>,request_id:u128,data:vector<u8>,ctx:&mut TxContext){
        let ticket= xcall::execute_call(xcall,dapp_state::get_xcall_cap(state),request_id,data,ctx);
        let msg= execute_ticket::message(&ticket);
        let from=execute_ticket::from(&ticket);

        if(msg==b"rollback"){
             xcall::execute_call_result(xcall,ticket,false,fee,ctx);

        }else if(msg==b"reply-response"){
             let reply_fee = coin::from_balance(balance::zero<SUI>(), ctx);
             send_message(state,xcall,reply_fee,network_address::to_string(&from),vector::empty<u8>(),vector::empty<u8>(),ctx);
             xcall::execute_call_result(xcall,ticket,true,fee,ctx);

        }else {
          xcall::execute_call_result(xcall,ticket,true,fee,ctx);
        };
    }

    entry public fun execute_rollback(state:&mut DappState,xcall:&mut XCallState, sn:u128,ctx: &mut TxContext){
         let ticket= xcall::execute_rollback(xcall,dapp_state::get_xcall_cap(state),sn,ctx);
        
         xcall::execute_rollback_result(xcall,ticket,true);

    }

    entry public fun add_connection(state:&mut DappState,net_id:String,source:vector<String>,destination:vector<String>,ctx:&mut TxContext){
        dapp_state::add_connection(state,net_id,source,destination,ctx);
    }

    entry public fun send_message(state:&DappState,xcall:&mut XCallState,fee: Coin<SUI>,to:String,data:vector<u8>,rollback:vector<u8>,ctx: &mut TxContext){
        let to=network_address::from_string(to);
        let connection= dapp_state::get_connection(state,network_address::net_id(&to));
        let sources=dapp_state::get_connection_source(&connection);
        let destinations=dapp_state::get_connection_dest(&connection);
        let envelope;
        if(rollback==vector::empty<u8>()){
            envelope =envelope::wrap_call_message(data,sources,destinations);
        } else {
            envelope =envelope::wrap_call_message_rollback(data,rollback,sources,destinations);
        };
        let envelope_bytes=envelope::encode(&envelope);
        xcall::send_call(xcall,fee,dapp_state::get_xcall_cap(state),network_address::to_string(&to),envelope_bytes,ctx);
    }



    #[test_only] use sui::test_scenario::{Self,Scenario};
    #[test_only]
    public fun init_dapp_state(admin:address,mut scenario:Scenario):Scenario{
     init(scenario.ctx());
     scenario.next_tx(admin);
     scenario
    }

    
    

    

    


}