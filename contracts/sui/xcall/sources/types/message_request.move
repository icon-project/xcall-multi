#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
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

    public fun decode(bytes:vector<u8>):CSMessageRequest {
        CSMessageRequest {
            from:network_address::from_string(string::utf8(b"abc")),
            to:network_address::from_string(string::utf8(b"def")),
            sn:0,
            message_type:0,
            data:vector::empty<u8>(),
            protocols:vector::empty<String>()
        }
    }

    public fun msg_type(req:&CSMessageRequest):u8 {
         req.message_type
    }

    public fun from(req:&CSMessageRequest):NetworkAddress {
        req.from
    }

    public fun to(req:&CSMessageRequest):NetworkAddress {
        req.to
    }

    public fun sn(req:&CSMessageRequest):u128 {
        req.sn
    }

    public fun data(req:&CSMessageRequest):vector<u8> {
        req.data
    }

    public fun protocols(req:&CSMessageRequest):vector<String> {
        req.protocols
    }

    public fun from_nid(req:&CSMessageRequest):String {
        network_address::net_id(&req.from)
    }
}