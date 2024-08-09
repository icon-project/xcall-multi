#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::network_address {
    use std::string::{Self, String};
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;
    use sui::hex;

   
   
    public struct NetworkAddress has drop,store,copy{
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
        let separator_index=string::index_of(&net_addr,&string::utf8(b"/"));
        let net_id=string::sub_string(&net_addr,0,separator_index);
        let addr=string::sub_string(&net_addr,separator_index+1,string::length(&net_addr));
        return NetworkAddress {
            net_id,
            addr,
        }
    }

    public fun net_id(self:&NetworkAddress):String {
        self.net_id
    }

     public fun addr(self:&NetworkAddress):String {
        self.addr
    }

    public fun to_string(self:&NetworkAddress):String {
        let mut addr=self.net_id;
        string::append(&mut addr,string::utf8(b"/"));
        string::append(&mut addr,self.addr);
        addr

    }

    public fun encode(self:&NetworkAddress):vector<u8>{
        encoder::encode_string(&to_string(self))
    }
    public fun decode(bytes:&vector<u8>):NetworkAddress {
        let network_address= decoder::decode_string(bytes);
        from_string(network_address)
    }
    public fun decode_raw(bytes:&vector<u8>):NetworkAddress {
        let value=decoder::decode(bytes);
        decode(&value)
    }
}

#[test_only]
module xcall::network_address_tests {

    use xcall::network_address::{Self};
    use std::string;

    #[test]
    fun test_network_address_encoding(){
        let address=network_address::create(string::utf8(b"0x1.ETH"),string::utf8(b"0xa"));
        let bytes= network_address::encode(&address);
        let expected= network_address::decode_raw(&bytes);
        assert!(address==expected,0x01);

    }

}