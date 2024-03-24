module xcall::call_message {
     use std::string::{Self, String};
use std::vector;
use std::option::{Self, Option,some,none};

const MSG_TYPE:u8=0;
      struct CallMessage has drop{
         data:vector<u8>
    }

     public fun msg_type():u8 {
         MSG_TYPE
    }
}