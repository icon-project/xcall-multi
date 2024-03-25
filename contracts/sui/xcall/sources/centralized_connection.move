#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::centralized_connection {
  use xcall::centralized_state::{Self,State};
  use std::string::{Self, String};
  use sui::bag::{Bag, Self};
  use sui::tx_context::{Self, TxContext};

    const PackageId:vector<u8> =b"centralized";

    public fun package_id_str():String {
        string::utf8(PackageId)
    }

    public fun connect():State{

      centralized_state::create()
    }

    public fun get_state(states:&Bag):&State {
      let package_id= package_id_str();
      let state:&State=bag::borrow(states,package_id);
      state
    }

    public fun get_fee(states:&Bag,netId:String,response:bool):u128{
      let state= get_state(states);
      centralized_state::get_fee(state,netId,response)

    }

    entry fun send_message(to:String,sn:u64,msg:vector<u8>,dir:u8,ctx: &mut TxContext){

    }

    entry fun receive_message(src:String,sn:u128,msg:vector<u8>,ctx: &mut TxContext){

    }


    entry fun claim_fees(ctx: &mut TxContext){

    }

    entry fun revert_message(sn:u128, ctx: &mut TxContext){}

    entry fun set_admin(addr:address, ctx: &mut TxContext){}

    entry fun set_fee(net_id:String,message_fee:u128,response_fee:u128, ctx: &mut TxContext){}

    

}