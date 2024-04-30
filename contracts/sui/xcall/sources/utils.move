#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::xcall_utils {
    use std::vector::length;
    use std::vector::borrow;
    use sui::vec_map::{Self, VecMap};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use std::string::{Self, String};
    use sui::hex;
   
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
        string::utf8(hex_bytes)
    }

    public fun id_from_hex_string(str: &String): ID {
        let bytes = str.bytes();
        let hex_bytes = hex::decode(*bytes);
        object::id_from_bytes(hex_bytes)
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
}