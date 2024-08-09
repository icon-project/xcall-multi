#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::call_message_rollback {
    use std::string::{Self, String};
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};

    const MSG_TYPE:u8=1;
    public struct CallMessageWithRollback has drop{
        data:vector<u8>,
        rollback:vector<u8>,
    }


    public fun create(data:vector<u8>,rollback:vector<u8>):CallMessageWithRollback{
        CallMessageWithRollback {
            data:data,
            rollback:rollback,
        }
    }

    public fun encode(self:CallMessageWithRollback):vector<u8>{
        let mut list=vector::empty<vector<u8>>();
        vector::push_back(&mut list,encoder::encode(&self.data));
        vector::push_back(&mut list,encoder::encode(&self.rollback));
        let encoded=encoder::encode_list(&list,false);
        encoded

    }
    
    public fun decode(bytes:&vector<u8>):CallMessageWithRollback{
        let decoded=decoder::decode_list(bytes);
        let data=  *vector::borrow(&decoded,0);
        let rollback=  *vector::borrow(&decoded,1);
        CallMessageWithRollback {
            data,
            rollback,

        }
    }

    public fun msg_type():u8 {
        MSG_TYPE
    }

    public fun rollback(self:&CallMessageWithRollback):vector<u8>{
        self.rollback
    }

    public fun data(self:&CallMessageWithRollback):vector<u8>{
        self.data
    }
}