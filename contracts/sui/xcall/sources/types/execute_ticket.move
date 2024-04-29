module xcall::execute_ticket {
    use xcall::network_address::{NetworkAddress};
 public struct ExecuteTicket {
        dapp_id:ID,
        request_id:u128,
        from:NetworkAddress,
        message:vector<u8>,
    }

    public(package) fun new(dapp_id:ID,request_id:u128,from:NetworkAddress,message:vector<u8>):ExecuteTicket{
        ExecuteTicket {
            dapp_id,
            request_id,
            from,
            message,
        }

    }

    public(package) fun consume(ticket:ExecuteTicket){
        let ExecuteTicket { dapp_id, request_id,from,message}=ticket;
    }
    public fun message(ticket:&ExecuteTicket):vector<u8>{
         ticket.message
    }

    public fun from(ticket:&ExecuteTicket):NetworkAddress {
        ticket.from
    }

    public fun request_id(ticket:&ExecuteTicket):u128 {
        ticket.request_id
    }

    public fun dapp_id(ticket:&ExecuteTicket):ID {
        ticket.dapp_id
    }

}