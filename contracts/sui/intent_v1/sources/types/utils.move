module intents_v1::utils {
    use std::string::{Self,String};
    use sui::hex::{Self};
    use sui::bcs::{Self};
    use sui::address::{Self as suiaddress};
    use std::type_name::{Self};

     public fun id_to_hex_string(id:&ID): String {
        let bytes = object::id_to_bytes(id);
        let hex_bytes = hex::encode(bytes);
        to_hex_string(string::utf8(hex_bytes))
      
        
    }
     public fun id_from_hex_string(str: &String): ID {
        let encoded = format_sui_address(str);
        let bytes = encoded.as_bytes();
        let hex_bytes = hex::decode(*bytes);
        object::id_from_bytes(hex_bytes)
    }
     public fun format_sui_address(addr: &String): String {
        let mut sui_addr = *addr;
        if (sui_addr.substring(0, 2) == string::utf8(b"0x")) {
            sui_addr = addr.substring(2, addr.length());
        };
        sui_addr
    }

     public fun address_to_hex_string(address:&address): String {
        let bytes = bcs::to_bytes(address);
        let hex_bytes = hex::encode(bytes);
        to_hex_string(string::utf8(hex_bytes))
    }

    public fun address_from_hex_string(str: &String): address {
        let mut modified_str = str;
        if(string::length(str) == 66 ){
            modified_str = &str.substring(2, 66);
        };
        let bytes = modified_str.as_bytes();
        let hex_bytes = hex::decode(*bytes);
        bcs::peel_address(&mut bcs::new(hex_bytes))
    }

    public fun address_from_str(val:&String):address{
        let addr_string=format_sui_address(val);
        suiaddress::from_ascii_bytes(addr_string.as_bytes())
    }

    public fun get_type_string<T>():String{
        to_hex_string(string::from_ascii(type_name::get<T>().into_string()))
    }

    public fun to_hex_string(str:String):String{
        let mut prefix = string::utf8(b"0x");
        prefix.append(str);
        prefix
    }

}