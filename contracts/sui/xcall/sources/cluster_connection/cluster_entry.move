module xcall::cluster_entry{

  use xcall::main::{Self as xcall};
  use xcall::xcall_state::{Self,Storage as XCallState,ConnCap};
  use xcall::cluster_state::{Self,get_state,get_state_mut, Validator};
  use std::string::{String};

  entry public fun receive_message(xcall:&mut XCallState,cap:&ConnCap,src_net_id:String,sn:u128,msg:vector<u8>,ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::check_save_receipt(state, src_net_id, sn);
      xcall::handle_message(xcall, cap,src_net_id, msg,ctx);
  }


  entry fun claim_fees(xcall:&mut XCallState,cap:&ConnCap,ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::claim_fees(state,ctx);
  }

  entry fun set_fee(xcall:&mut XCallState,cap:&ConnCap,net_id:String,message_fee:u64,response_fee:u64, ctx: &TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::set_fee(state,net_id,message_fee,response_fee,ctx.sender());
  }

  entry fun get_receipt(states: &XCallState,connection_id:String,net_id:String,sn:u128,_ctx: &TxContext):bool{
      let state = get_state(states.get_connection_states(),connection_id);
      cluster_state::get_receipt(state,net_id,sn)
  }

  entry fun get_fee(states: &XCallState,connection_id:String,net_id:String,response:bool,_ctx: &TxContext):u64{
      let state = get_state(states.get_connection_states(),connection_id);
      cluster_state::get_fee(state,&net_id,response)
  }

  entry fun add_validator(xcall:&mut XCallState,cap:&ConnCap,validator_pubkey:String,_ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::add_validator(state,validator_pubkey);
  }

  entry fun remove_validator(xcall:&mut XCallState,cap:&ConnCap,validator_pubkey:String,ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::remove_validator(state,validator_pubkey,ctx);
  }

  entry fun set_validator_threshold(xcall:&mut XCallState,cap:&ConnCap,threshold:u64,_ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::set_validator_threshold(state,threshold);
  }

  entry fun get_validators(states: &XCallState,connection_id:String,_ctx: &TxContext):vector<Validator>{
      let state = get_state(states.get_connection_states(),connection_id);
      cluster_state::get_validators(state)
  }

  entry fun get_validators_threshold(states: &XCallState,connection_id:String,_ctx: &TxContext):u64{
      let state = get_state(states.get_connection_states(),connection_id);
      cluster_state::get_validator_threshold(state)
  }

  entry fun recieve_message_with_signatures(xcall:&mut XCallState,cap:&ConnCap,src_net_id:String,sn:u128,msg:vector<u8>,signatures:vector<vector<u8>>,ctx: &mut TxContext){
      let state=get_state_mut(xcall_state::get_connection_states_mut(xcall),cap.connection_id());
      cluster_state::verify_signatures(state, msg,signatures);
      cluster_state::check_save_receipt(state, src_net_id, sn);
      xcall::handle_message(xcall, cap,src_net_id, msg,ctx);
  }
}