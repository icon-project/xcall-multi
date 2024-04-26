#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::centralized_connection {
  use xcall::centralized_state::{Self,State, ReceiptKey};
  use std::string::{Self, String};
  use sui::bag::{Bag, Self};
  use sui::tx_context;
  use sui::event;
  use sui::table::{Self, Table};
  use sui::sui::SUI;
  use sui::coin::{Self, Coin};
  use sui::balance;
  use xcall::xcall_utils::{Self as utils};

  const ENotEnoughFee: u64 = 10;


    /* ================= events ================= */

  public struct Message has copy, drop {
      to:String,
      conn_sn:u128,
      msg:vector<u8>,
     
  }


    const PackageId:vector<u8> =b"centralized";

    public fun package_id_str():String {
        string::utf8(PackageId)
    }

    public fun connect():State{

      centralized_state::create()
    }

    public fun get_state(states:&mut Bag):&mut State {
      let package_id= package_id_str();
      let state:&mut State=bag::borrow_mut(states,package_id);
      state
    }

    public fun get_fee(states:&mut Bag,netId:String,response:bool):u64{
      let state= get_state(states);
      centralized_state::get_fee(state,&netId,response)

    }

    fun get_next_connection_sn(state:&mut State):u128 {
        let sn = centralized_state::get_next_conn_sn(state);
        sn
      
    }

    entry public(package) fun send_message(states:&mut Bag,coin:&mut Coin<SUI>,to:String,sn:u128,msg:vector<u8>,response:bool,ctx: &mut TxContext){
      let mut state= get_state(states);
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

    entry fun receive_message(states: &mut Bag,src:String,sn:u128,msg:vector<u8>,ctx: &mut TxContext){
      let state = get_state(states);
      centralized_state::check_duplicate_message(state, src, sn);
      
      // xcall::handle_message(&self.xcall, src_network, msg);

    }


    entry fun claim_fees(ctx: &mut TxContext){
        // transfer::public_transfer(ctx.coin, self.admin);
    }

    entry fun revert_message(sn:u128, ctx: &mut TxContext){
        // xcall::handle_error(&self.xcall, sn);
    }

    entry fun set_admin(addr:address, ctx: &mut TxContext){}

    entry fun set_fee(states: &mut Bag,net_id:String,message_fee:u64,response_fee:u64, ctx: &mut TxContext){
      let state = get_state(states);
      centralized_state::set_fee(state,net_id,message_fee,response_fee);
    }

    entry fun get_receipt(states: &mut Bag,net_id:String,sn:u128,ctx: &mut TxContext):bool{
      let state = get_state(states);
      centralized_state::get_receipt(state,net_id,sn)
    }

    

}

