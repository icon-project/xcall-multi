module xcall::centralized_receive {

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


    entry public fun receive_message(xcall:&mut XCallState,src:String,sn:u128,msg:vector<u8>,ctx: &mut TxContext){
      centralized_state::check_save_receipt(get_state(xcall_state::get_connection_states_mut(xcall)), src, sn);
      let cap:ConnCap=* get_state(xcall_state::get_connection_states_mut(xcall)).conn_cap();
      xcall::handle_message(xcall, &cap,src, msg,ctx);
    }

   



}