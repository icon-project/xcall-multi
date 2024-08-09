module xcall::rollback_ticket {
    public struct RollbackTicket has drop {
        rollback:vector<u8>,
        sn:u128,
        dapp_id:ID,

    }

    public(package) fun new(sn:u128,rollback:vector<u8>,dapp_id:ID):RollbackTicket{
        RollbackTicket {
            rollback,
            sn,
            dapp_id
        }
    }

    public fun rollback(self:&RollbackTicket):vector<u8>{
        self.rollback
    }

    public fun sn(self:&RollbackTicket):u128 {
        self.sn
    }

    public fun dapp_id(self:&RollbackTicket):ID {
        self.dapp_id
    }

    public(package) fun consume(self:RollbackTicket){
        let RollbackTicket { rollback, sn,dapp_id}=self;
    }


}