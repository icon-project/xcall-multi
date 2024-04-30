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

    public(package) fun consume(self:ExecuteTicket){
       let ExecuteTicket { dapp_id, request_id,from,message}=self;
    }
    public fun message(self:&ExecuteTicket):vector<u8>{
         self.message
    }

    public fun from(self:&ExecuteTicket):NetworkAddress {
        self.from
    }

    public fun request_id(self:&ExecuteTicket):u128 {
        self.request_id
    }

    public fun dapp_id(self:&ExecuteTicket):ID {
        self.dapp_id
    }

}