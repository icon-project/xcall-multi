#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment,implicit_const_copy)]
module xcall::connections{
    use std::string::{Self, String};
    use sui::bag::{Bag, Self};
    use xcall::centralized_connection::{Self};
    use xcall::cluster_connection::{Self};
    use xcall::cluster_state_optimized::{Self,State,create_admin_cap};
    use xcall::xcall_state::{ConnCap};
    use sui::coin::{Self,Coin};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    


    const EConnectionNotFound:u64=0;
    const ConnCentralized:vector<u8> =b"centralized";
    const ConnCluster:vector<u8> =b"cluster";


    public(package) fun register(states:&mut Bag,connection_id:String,ctx:&mut TxContext){
        
        if (get_connection_type(&connection_id).as_bytes()==ConnCentralized){
            let state= centralized_connection::connect();
            bag::add(states, connection_id, state);
        }else
        if (get_connection_type(&connection_id).as_bytes()==ConnCluster){
            let state= cluster_connection::connect(ctx);
            let admin_cap=cluster_state_optimized::create_admin_cap(connection_id,ctx);
            transfer::public_transfer(admin_cap, ctx.sender());
            bag::add(states, connection_id, state);
        }else{
            abort EConnectionNotFound
        }
        
    }

    public(package) fun get_fee(states:&Bag,connection_id:String,netId:String,response:bool):u64{

        if (get_connection_type(&connection_id).as_bytes()==ConnCentralized){
            let fee= centralized_connection::get_fee(states,connection_id,netId,response);
            fee
        }else
        if (get_connection_type(&connection_id).as_bytes()==ConnCluster){
            let fee= cluster_connection::get_fee(states,connection_id,netId,response);
            fee
        }else{
            abort EConnectionNotFound
        }
    }

    public(package) fun send_message(states:&mut Bag,connection_id:String,coin:Coin<SUI>,netId:String,sn:u128,msg:vector<u8>,is_response:bool,ctx:&mut TxContext){

        if (get_connection_type(&connection_id).as_bytes()==ConnCentralized){
            centralized_connection::send_message(states,connection_id,coin,netId,sn,msg,is_response,ctx);
        }else
        if (get_connection_type(&connection_id).as_bytes()==ConnCluster){
            cluster_connection::send_message(states,connection_id,coin,netId,sn,msg,is_response,ctx);
        }else{
            abort EConnectionNotFound
        }
    }

    fun get_connection_type(connection_id:&String):String{
        let separator_index=string::index_of(connection_id,&string::utf8(b"-"));
        let connType=string::substring(connection_id,0,separator_index);
        connType
    }
    
}