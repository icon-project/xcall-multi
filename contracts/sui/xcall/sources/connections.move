#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::connections{
use std::string::{Self, String};
use sui::bag::{Bag, Self};
use xcall::centralized_connection::{Self};
use xcall::centralized_state::{Self,State};


const EConnectionNotFound:u64=0;

const ConnCentralized:vector<u8> =b"centralized";


    public fun register(states:&mut Bag,package_id:String){
       
        if (package_id==centralized_connection::package_id_str()){
              let state= centralized_connection::connect();
              bag::add(states, package_id, state);

        }else{
           abort EConnectionNotFound
        }
       
        
    }

    public fun get_fee(states:&mut Bag,package_id:String,netId:String,response:bool):u64{

         if (package_id==centralized_connection::package_id_str()){
            let fee= centralized_connection::get_fee(states,netId,response);
            fee
              
             
        }else{
           abort EConnectionNotFound
        }
       

    }
    
}