#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::call_message {
     use std::string::{Self, String};
     use sui_rlp::encoder::{Self};
     use sui_rlp::decoder::{Self};

     const MSG_TYPE:u8=0;

     public struct CallMessage has drop{
          data:vector<u8>
     }

     public fun new(data:vector<u8>):CallMessage{
          CallMessage {
            data,
          }
     }

     public fun msg_type():u8 {
          MSG_TYPE
     }

     public fun encode(self:CallMessage):vector<u8>{
          let mut list=vector::empty<vector<u8>>();
          vector::push_back(&mut list,encoder::encode(&self.data));

          let encoded=encoder::encode_list(&list,false);
          encoded

     }

     public fun decode(bytes:&vector<u8>):CallMessage{
          let decoded=decoder::decode_list(bytes);
          let data=  *vector::borrow(&decoded,0);

          CallMessage {
            data,
          }
     }
}