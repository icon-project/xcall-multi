#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::cs_message {
    use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::message_request::{Self,CSMessageRequest};
     use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;

    public struct CSMessage has store,drop{
        msg_type:u8,
        payload:vector<u8>,
    }

    public fun from_message_request(req:CSMessageRequest):CSMessage {
        CSMessage {
            msg_type:message_request::msg_type(&req),
            payload:message_request::encode(&req),
        }
    }

    public fun decode(bytes:vector<u8>):CSMessage {
        CSMessage {
            msg_type:0,
            payload:vector::empty<u8>(),
        }
    }

    public fun msg_type( msg:&CSMessage):u8 {
        msg.msg_type
    }

    public fun payload( msg:&CSMessage):vector<u8> {
        msg.payload
    }
}