#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::cs_message {
    use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::message_request::{Self,CSMessageRequest};
     use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;

    struct CSMessage has store{
        msg_type:u8,
        payload:vector<u8>,
    }

    public fun from_message_request(req:CSMessageRequest):CSMessage {
        CSMessage {
            msg_type:message_request::msg_type(&req),
            payload:message_request::encode(req),
        }
    }
}