#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::persistent_message {
use std::string::{Self, String};

const MSG_TYPE:u8=2;
      public struct PersistentMessage has drop{
         data:vector<u8>
    }

    public fun new(data:vector<u8>):PersistentMessage{
      PersistentMessage {
          data,
      }
    }

    public fun get_data(self: &PersistentMessage){
        self.data;
    }

     public fun msg_type():u8 {
         MSG_TYPE
    }
}