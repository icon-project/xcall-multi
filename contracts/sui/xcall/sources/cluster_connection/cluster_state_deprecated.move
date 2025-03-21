module xcall::cluster_state {
    use std::string::{String};
    use sui::vec_map::{Self, VecMap};
    use xcall::xcall_utils::{Self as utils};
    use sui::coin::{Self};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    use sui::bag::{Bag, Self};
    

    //EVENTS
    public struct ValidatorSetAdded has copy, drop {
        validators: vector<vector<u8>>,
        threshold: u64
    }

    public struct AdminCap has key,store {
        id: UID,
        connection_id: String
    }

    public fun get_state(states:&Bag,connection_id:String):&State {
      let state:&State=bag::borrow(states,connection_id);
      state
    }
    

    public struct ReceiptKey has copy, drop, store {
        conn_sn: u128,
        nid: String,
    }

    public struct State has store{ 
        message_fee: VecMap<String, u64>,
        response_fee: VecMap<String, u64>,
        receipts: VecMap<ReceiptKey, bool>,
        conn_sn: u128,
        balance: Balance<SUI>,
        validators: vector<vector<u8>>,
        validators_threshold:u64,

    }

    public fun get_fee(self: &State, netId: &String, need_response: bool): u64 {
        0
    }
   
}
