module intents_v1::utils {
    use std::string::{Self,String};
    use sui::hex::{Self};
    use sui::bcs::{Self};

     public fun id_to_hex_string(id:&ID): String {
        let bytes = object::id_to_bytes(id);
        let hex_bytes = hex::encode(bytes);
        let mut prefix = string::utf8(b"0x");
        prefix.append(string::utf8(hex_bytes));
        prefix
    }
     public fun id_from_hex_string(str: &String): ID {
        let encoded = format_sui_address(str);
        let bytes = encoded.bytes();
        let hex_bytes = hex::decode(*bytes);
        object::id_from_bytes(hex_bytes)
    }
     public fun format_sui_address(addr: &String): String {
        let mut sui_addr = *addr;
        if (sui_addr.sub_string(0, 2) == string::utf8(b"0x")) {
            sui_addr = addr.sub_string(2, addr.length());
        };
        sui_addr
    }

     public fun address_to_hex_string(address:&address): String {
        let bytes = bcs::to_bytes(address);
        let hex_bytes = hex::encode(bytes);
        let mut prefix = string::utf8(b"0x");
        prefix.append(string::utf8(hex_bytes));
        prefix
    }

    public fun address_from_hex_string(str: &String): address {
        let mut modified_str = str;
        if(string::length(str) == 66 ){
            modified_str = &str.substring(2, 66);
        };
        let bytes = modified_str.bytes();
        let hex_bytes = hex::decode(*bytes);
        bcs::peel_address(&mut bcs::new(hex_bytes))
    }

}