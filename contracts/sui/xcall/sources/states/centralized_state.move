#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::centralized_state {
    use sui::vec_map::{Self, VecMap};
     use std::string::{Self, String};
     use xcall::xcall_utils::{Self};
    /**
     message_fee: Map<'a, NetId, u128>,
    response_fee: Map<'a, NetId, u128>,
    admin: Item<'a, Addr>,
    conn_sn: Item<'a, u128>,
    receipts: Map<'a, (String, u128), bool>,
    xcall: Item<'a, Addr>,
    */
    struct State has store {
        message_fee:VecMap<String,u128>,
        response_fee:VecMap<String,u128>,
        connection_sn:u128,
        receipts:VecMap<String,u128>,
       
    }

    public fun create():State{
        State {
            message_fee:vec_map::empty<String,u128>(),
            response_fee:vec_map::empty<String,u128>(),
            connection_sn:0,
            receipts:vec_map::empty<String,u128>(),

        }
    }

    public fun get_fee(self:&State,netId:String,response:bool):u128 {
       let fee:u128=  if(response==true){
            xcall_utils::get_or_default(&self.message_fee,&netId,0)+xcall_utils::get_or_default(&self.response_fee,&netId,0)
        }else {
           xcall_utils::get_or_default(&self.message_fee,&netId,0)
        };
        fee
        
    }

    public fun get_next_sn(self:&mut State):u128 {
        let sn=self.connection_sn+1;
        self.connection_sn=sn;
        sn
    }


}