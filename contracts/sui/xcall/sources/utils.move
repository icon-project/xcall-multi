#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::xcall_utils {
    use std::vector::length;
    use std::vector::borrow;
    use sui::vec_map::{Self, VecMap};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self};
    use std::string::{Self, String};
    use sui::hex;
    use sui::bcs::{Self};
   
   public fun are_equal<Element>(a1:&vector<Element>,a2:&vector<Element>): bool {

       if(length(a1)!=length(a2)){
            false
       }else{
         let mut i = 0;
        let len = length(a1);
        while (i < len) {
            if (borrow(a1, i) != borrow(a2,i)) return false;
            i = i + 1;
        };
        true
        }   
    }


    public fun id_to_hex_string(id:&ID): String {
        let bytes = object::id_to_bytes(id);
        let hex_bytes = hex::encode(bytes);
        let mut prefix = string::utf8(b"0x");
        prefix.append(string::utf8(hex_bytes));
        prefix
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
            modified_str = &str.sub_string(2, 66);
        };
        let bytes = modified_str.bytes();
        let hex_bytes = hex::decode(*bytes);
        bcs::peel_address(&mut bcs::new(hex_bytes))
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

    public fun destroy_or_transfer_balance<T>(balance: Balance<T>, recipient: address, ctx: &mut TxContext) {
        if (balance::value(&balance) == 0) {
            balance::destroy_zero(balance);
            return
        };
        transfer::public_transfer(
            coin::from_balance(balance, ctx),
            recipient
        );
    }

    public fun get_or_default<K: copy, V: copy+drop>(self: &VecMap<K,V>, key: &K,default:V): V {
       let value= if (vec_map::contains(self, key)) {
            *vec_map::get(self, key)
        } else {
            default
        };
        value
    }

    #[test_only] use sui::test_scenario::{Self,Scenario};
    #[test]
    fun test_id_hex(){
        let admin = @0xBABE;
        let mut scenario = test_scenario::begin(admin);
        let uid = object::new(scenario.ctx());
        let id = object::uid_to_inner(&uid);

        let id_hex = id_to_hex_string(&id);

        let id_from_hex = id_from_hex_string(&id_hex);

        assert!(id_from_hex==id, 0x01);
        object::delete(uid);
        scenario.end();
    }

    #[test]
    fun test_format_sui_address(){
        let admin = @0xBABE;
        let mut scenario = test_scenario::begin(admin);
        let uid = object::new(scenario.ctx());
        let id = object::uid_to_inner(&uid);

        let id_hex = id_to_hex_string(&id);

        let formatted1 = format_sui_address(&id_hex);

        let mut prefix = b"0x".to_string();
        prefix.append(id_hex);
        let formatted2 = format_sui_address(&prefix);

        let id_from_hex1 = id_from_hex_string(&formatted1);
        let id_from_hex2 = id_from_hex_string(&formatted2);

        assert!(id_from_hex1==id_from_hex2 && id_from_hex1==id, 0x01);
        object::delete(uid);
        scenario.end();
    }


}