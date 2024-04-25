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

     public fun message(ticket:&RollbackTicket):vector<u8>{
         ticket.rollback
    }

   

    public fun sn(ticket:&RollbackTicket):u128 {
        ticket.sn
    }

    public fun dapp_id(ticket:&RollbackTicket):ID {
        ticket.dapp_id
    }

    public(package) fun consume(ticket:RollbackTicket){
        let RollbackTicket { rollback, sn,dapp_id}=ticket;
    }


}