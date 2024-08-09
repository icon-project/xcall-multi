module xcall::centralized_entry{

  use xcall::main::{Self as xcall};
  use xcall::xcall_state::{Self,Storage as XCallState,ConnCap};
  use xcall::centralized_state::{Self,get_state,get_state_mut};
  use std::string::{String};

  entry public fun receive_message(xcall:&mut XCallState,cap:&ConnCap,src_net_id:String,sn:u128,msg:vector<u8>,ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      centralized_state::check_save_receipt(state, src_net_id, sn);
      xcall::handle_message(xcall, cap,src_net_id, msg,ctx);
  }


  entry fun claim_fees(xcall:&mut XCallState,cap:&ConnCap,ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      centralized_state::claim_fees(state,ctx);
  }

  entry fun set_fee(xcall:&mut XCallState,cap:&ConnCap,net_id:String,message_fee:u64,response_fee:u64, ctx: &TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      centralized_state::set_fee(state,net_id,message_fee,response_fee,ctx.sender());
  }

  entry fun get_receipt(states: &XCallState,connection_id:String,net_id:String,sn:u128,_ctx: &TxContext):bool{
      let state = get_state(states.get_connection_states(),connection_id);
      centralized_state::get_receipt(state,net_id,sn)
  }

  entry fun get_fee(states: &XCallState,connection_id:String,net_id:String,response:bool,_ctx: &TxContext):u64{
      let state = get_state(states.get_connection_states(),connection_id);
      centralized_state::get_fee(state,&net_id,response)
  }



   



}