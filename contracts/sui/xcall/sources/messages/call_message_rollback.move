#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::call_message_rollback {
     use std::string::{Self, String};
use std::vector;
use std::option::{Self, Option,some,none};
    const MSG_TYPE:u8=1;
    struct CallMessageWithRollback has drop{
        data:vector<u8>,
        rollback:vector<u8>,
     }

     public fun encode(msg:CallMessageWithRollback):vector<u8>{
         vector::empty<u8>()
    }

    public fun create(data:vector<u8>,rollback:vector<u8>):CallMessageWithRollback{
         CallMessageWithRollback {
            data:data,
            rollback:rollback,
        }
    }

  

     public fun decode(bytes:vector<u8>):CallMessageWithRollback{
          CallMessageWithRollback {
           data:vector::empty<u8>(),
           rollback:vector::empty<u8>(),

         }
    }

    public fun msg_type():u8 {
         MSG_TYPE
    }

    public fun rollback(msg:CallMessageWithRollback):vector<u8>{
        msg.rollback
    }
}