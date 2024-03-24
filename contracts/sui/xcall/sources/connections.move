module xcall::connections{
use std::string::{Self, String};
use sui::bag::{Bag, Self};
use xcall::centralized_connection::{Self};


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

    
}