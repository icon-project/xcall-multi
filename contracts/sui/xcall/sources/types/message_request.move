module xcall::message_request {
use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
    use std::vector;



     struct CSMessageRequest has store,drop{
    from:NetworkAddress,
    to: NetworkAddress,
    sn:u128,
    message_type:u8,
    data:vector<u8>,
    protocols:vector<String>,
   }


    public fun create(from:NetworkAddress,
    to: NetworkAddress,
    sn:u128,
    message_type:u8,
    data:vector<u8>,
    protocols:vector<String>):CSMessageRequest {
        CSMessageRequest {
            from:from,
            to:to,
            sn:sn,
            message_type:message_type,
            data:data,
            protocols:protocols
        }


    }

    public fun encode(req:CSMessageRequest):vector<u8>{
           vector::empty<u8>()
    }

    public fun msg_type(req:&CSMessageRequest):u8 {
         req.message_type
    }


}