#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::utils {
    use std::vector::length;
    use std::vector::borrow;
    use sui::vec_map::{Self, VecMap};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
   
   public fun are_equal<Element>(a1:&vector<Element>,a2:&vector<Element>): bool {

       if(length(a1)!=length(a2)){
            false
       }else{
         let i = 0;
        let len = length(a1);
        while (i < len) {
            if (borrow(a1, i) != borrow(a2,i)) return false;
            i = i + 1;
        };
        true

       }

     

       

    
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
}