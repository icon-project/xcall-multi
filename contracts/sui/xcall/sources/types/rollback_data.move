#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::rollback_data {
    use std::string::{Self, String};
    use xcall::network_address::{Self,NetworkAddress};
    public struct RollbackData has store,drop, copy{
        from:ID,
        to:String,
        sources:vector<String>,
        rollback:vector<u8>,
        enabled:bool, 
    }

     public fun create(from:ID,to:String,sources:vector<String>,rollback:vector<u8>,enabled:bool):RollbackData{
        RollbackData {
            from:from,
            to:to,
            sources:sources,
            rollback:rollback,
            enabled:enabled
        }  
    }

    public fun enabled(self:&RollbackData):bool{
        self.enabled
    }

    public fun from(self:&RollbackData):ID{
        self.from
    }

    public fun to(self:&RollbackData):String{
        self.to
    }

    public fun sources(self:&RollbackData):vector<String>{
        self.sources
    }

    public fun rollback(self:&RollbackData):vector<u8>{
        self.rollback
    }

    public(package) fun enable_rollback(self:&mut RollbackData){

        self.enabled = true
    }
}