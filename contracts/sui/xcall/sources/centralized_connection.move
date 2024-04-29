#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::centralized_connection {
  use xcall::centralized_state::{Self,State, ReceiptKey,get_state};
  use std::string::{Self, String};
  use sui::bag::{Bag, Self};
  use sui::tx_context;
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
     
  }


    

    public(package) fun connect(cap:ConnCap,admin:address):State{

      centralized_state::create(cap,admin)
    }

    public fun get_fee(states:&mut Bag,netId:String,response:bool):u64{
      let state= get_state(states);
      centralized_state::get_fee(state,&netId,response)

    }

     fun get_next_connection_sn(state:&mut State):u128 {
        let sn = centralized_state::get_next_conn_sn(state);
        sn
      
    }

     public(package) fun send_message(states:&mut Bag,coin:&mut Coin<SUI>,to:String,sn:u128,msg:vector<u8>,response:bool,ctx: &mut TxContext){
      let state= get_state(states);
      let fee = if (sn==0) {
        centralized_state::get_fee(state,&to,false)
        } else {
         centralized_state::get_fee(state,&to,response)
        };
       
      assert!(coin.value() > fee, ENotEnoughFee);
      let paid= coin.split(fee,ctx);
      let paid_balance=paid.into_balance();
      centralized_state::deposit(state,paid_balance);
      let conn_sn = get_next_connection_sn(state);
      event::emit(Message {
            to,
            conn_sn,
            msg,
        });
    }

  


    

    entry fun revert_message(sn:u128, ctx: &mut TxContext){
        // xcall::handle_error(&self.xcall, sn);
    }

    

    

    entry fun get_receipt(states: &mut Bag,net_id:String,sn:u128,ctx: &mut TxContext):bool{
      let state = get_state(states);
      centralized_state::get_receipt(state,net_id,sn)
    }

    

}

