#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::centralized_connection {
  use xcall::centralized_state::{Self,State, ReceiptKey,get_state,get_state_mut};
  use std::string::{Self, String};
  use sui::bag::{Bag, Self};
  use sui::event;
  use sui::table::{Self, Table};
  use sui::sui::SUI;
  use sui::coin::{Self, Coin};
  use sui::balance;
  use xcall::xcall_utils::{Self as utils};
  use xcall::xcall_state::{Self,ConnCap};
  use xcall::xcall_state::{Storage as XCallState};

  const ENotEnoughFee: u64 = 10;


  /* ================= events ================= */

  public struct Message has copy, drop {
    to:String,
    conn_sn:u128,
    msg:vector<u8>,
    // same package can instantiate multiple connection so this is required
    connection_id:String,

  }
  
  public(package) fun connect():State{
    centralized_state::create()
  }

  public fun get_fee(states:&Bag,connection_id:String,netId:String,response:bool):u64{
    let state= get_state(states,connection_id);
    centralized_state::get_fee(state,&netId,response)
  }

  public(package) fun get_next_connection_sn(state:&mut State):u128 {
    let sn = centralized_state::get_next_conn_sn(state);
    sn
  }
  // this is safe because only package can call this other xcall will call other deployed instance
  public(package) fun send_message(states:&mut Bag,connection_id:String,coin:Coin<SUI>,to:String,sn:u128,msg:vector<u8>,is_response:bool,ctx: &mut TxContext){
    let mut fee = 0;
    if(!is_response){
      fee = get_fee(states,connection_id, to, sn>0);
    };
    assert!(coin.value() >= fee, ENotEnoughFee);
    let balance = coin.into_balance();
    centralized_state::deposit(get_state_mut(states,connection_id),balance);
    let conn_sn = get_next_connection_sn(get_state_mut(states,connection_id));

    event::emit(Message {
    to,
    conn_sn,
    msg,
    connection_id
    });
  }

}

