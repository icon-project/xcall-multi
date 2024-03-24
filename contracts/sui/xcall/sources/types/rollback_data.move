module xcall::rollback_data {
    use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
    struct RollbackData has store,drop{
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