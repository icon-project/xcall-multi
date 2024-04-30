#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::envelope{

use std::string::{Self, String};
use std::option::{some,none};
use xcall::call_message::{Self};
use xcall::call_message_rollback::{Self};
use xcall::persistent_message::{Self};
use sui_rlp::encoder;
use sui_rlp::decoder;

 public struct XCallEnvelope has drop{
        message_type:u8,
        message:vector<u8>,
        sources:vector<String>,
        destinations:vector<String>,
    }

      public fun encode(req:&XCallEnvelope):vector<u8>{
          let mut list=vector::empty<vector<u8>>();
           vector::push_back(&mut list,encoder::encode_u8(req.message_type));
          vector::push_back(&mut list,encoder::encode(&req.message));
          vector::push_back(&mut list,encoder::encode_strings(&req.sources));
          vector::push_back(&mut list,encoder::encode_strings(&req.destinations));

          let encoded=encoder::encode_list(&list,false);
          encoded
    }

    public fun decode(bytes:&vector<u8>):XCallEnvelope {
         let decoded=decoder::decode_list(bytes);
         let message_type= decoder::decode_u8(vector::borrow(&decoded,0));
         let message=  *vector::borrow(&decoded,1);
        
         let sources= decoder::decode_strings(vector::borrow(&decoded,2));
         let destinations= decoder::decode_strings(vector::borrow(&decoded,3));
         let req=XCallEnvelope {
            message_type,
            message,
            sources,destinations
         };
         req

    }

     public fun wrap_call_message(data:vector<u8>,sources:vector<String>,destinations:vector<String>): XCallEnvelope {
        let envelope= XCallEnvelope {
            message_type:call_message::msg_type(),
            message:data,
            sources:sources,
            destinations:destinations,

        };
        envelope

    }

    public fun wrap_persistent_message(data:vector<u8>,sources:vector<String>,destinations:vector<String>): XCallEnvelope {
        let envelope= XCallEnvelope {
            message_type:persistent_message::msg_type(),
            message:data,
            sources:sources,
            destinations:destinations,

        };
        envelope

    }

     public fun wrap_call_message_rollback(data:vector<u8>,rollback:vector<u8>,sources:vector<String>,destinations:vector<String>): XCallEnvelope {
        let message= call_message_rollback::create(
            data,
            rollback,
        );
        let envelope= XCallEnvelope {
            message_type:call_message_rollback::msg_type(),
            message:call_message_rollback::encode(message),
            sources:sources,
            destinations:destinations,

        };
        envelope

    }

    public fun rollback(self:&XCallEnvelope):Option<vector<u8>>{
        if (self.message_type==call_message_rollback::msg_type()) {
            let msg= call_message_rollback::decode(&self.message);
             some(call_message_rollback::rollback(&msg))

        }else {
         none()
        }
             
    }

    public fun sources(self:&XCallEnvelope):vector<String>{
        self.sources
    }

    public fun destinations(self:&XCallEnvelope):vector<String>{
        self.destinations
    }

    public fun msg_type(self:&XCallEnvelope):u8 {
        self.message_type
    }
    public fun message(self:&XCallEnvelope):vector<u8>{
        self.message
    }

}