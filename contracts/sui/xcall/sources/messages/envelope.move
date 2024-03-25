#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::envelope{
    use std::string::{Self, String};
use std::vector;
use std::option::{Self, Option,some,none};
use xcall::call_message::{Self};
use xcall::call_message_rollback::{Self};

 struct XCallEnvelope has drop{
        message_type:u8,
        message:vector<u8>,
        sources:vector<String>,
        destinations:vector<String>,
    }

      public fun encode(msg:XCallEnvelope):vector<u8>{
         vector::empty<u8>()
    }

     public fun decode(bytes:vector<u8>):XCallEnvelope{
          XCallEnvelope {
            message_type:1,
            message:vector::empty<u8>(),
            sources:vector::empty<String>(),
            destinations:vector::empty<String>(),

         }
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
            let msg= call_message_rollback::decode(self.message);
             some(call_message_rollback::rollback(msg))

        }else {
         none()
        }
             
    }

    public fun sources(self:&XCallEnvelope):vector<String>{
        self.sources
    }

    public fun msg_type(self:&XCallEnvelope):u8 {
        self.message_type
    }
    public fun message(self:&XCallEnvelope):vector<u8>{
        self.message
    }

}