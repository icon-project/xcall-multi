module xcall::centralized_entry{

use xcall::main::{Self as xcall};
  use xcall::xcall_state::{Self,Storage as XCallState,ConnCap};
  use xcall::centralized_state::{Self,State,get_state};
  use std::string::{Self, String};
  use sui::bag::{Bag, Self};
  use sui::tx_context;
  use sui::event;
  use sui::table::{Self, Table};
  use sui::sui::SUI;
  use sui::coin::{Self, Coin};
  use sui::balance;


    entry public fun receive_message(xcall:&mut XCallState,src_net_id:String,sn:u128,msg:vector<u8>,ctx: &mut TxContext){
      centralized_state::check_save_receipt(get_state(xcall_state::get_connection_states_mut(xcall)), src_net_id, sn);
      let cap:ConnCap=* get_state(xcall_state::get_connection_states_mut(xcall)).conn_cap();
      xcall::handle_message(xcall, &cap,src_net_id, msg,ctx);
    }

    entry fun set_admin(xcall:&mut XCallState,addr:address, ctx: &TxContext){
      let state=get_state(xcall_state::get_connection_states_mut(xcall));
      state.set_admin(addr,ctx.sender());

    }

    entry fun claim_fees(xcall:&mut XCallState,ctx: &mut TxContext){
      let state=get_state(xcall_state::get_connection_states_mut(xcall));
      centralized_state::claim_fees(state,ctx.sender(),ctx);
    }

    entry fun set_fee(xcall:&mut XCallState,net_id:String,message_fee:u64,response_fee:u64, ctx: &TxContext){
      let state=get_state(xcall_state::get_connection_states_mut(xcall));
      centralized_state::set_fee(state,net_id,message_fee,response_fee,ctx.sender());
    }



   



}