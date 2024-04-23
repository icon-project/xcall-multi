#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::rollback_data {
    use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
     use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;
    public struct RollbackData has store,drop{
        from:NetworkAddress,
        to:NetworkAddress,
        sources:vector<String>,
        rollback:vector<u8>,
        enabled:bool, 
    }

     public fun create(from:NetworkAddress,to:NetworkAddress,sources:vector<String>,rollback:vector<u8>,enabled:bool):RollbackData{
        RollbackData {
            from:from,
            to:to,
            sources:sources,
            rollback:rollback,
            enabled:enabled
        }

        
    }
}