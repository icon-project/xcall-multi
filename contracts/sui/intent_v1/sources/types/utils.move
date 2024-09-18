module intents_v1::utils {
    use std::string::{Self,String};
    use sui::hex::{Self};

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
}