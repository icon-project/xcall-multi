#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::network_address {
      use std::string::{Self, String};
    use sui::object::{Self, ID, UID};

   
   
    struct NetworkAddress has drop,store,copy{
        net_id:String,
        addr:String,
    }

     public fun create(nid:String,addr:String):NetworkAddress {
        return NetworkAddress{
            net_id:nid,
            addr:addr,
        }
    }

    public fun from_string(net_addr:String):NetworkAddress {
        return NetworkAddress {
            net_id:string::utf8(b"nid"),
            addr:string::utf8(b"addr"),
        }
    }

    public fun net_id(self:&NetworkAddress):String {
        self.net_id
    }

     public fun addr(self:&NetworkAddress):String {
        self.addr
    }
}