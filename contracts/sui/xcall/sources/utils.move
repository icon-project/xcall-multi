#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::utils {
    use std::vector::length;
    use std::vector::borrow;
    use sui::vec_map::{Self, VecMap};
   
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

  public fun get_or_default<K: copy, V: copy+drop>(self: &VecMap<K,V>, key: &K,default:V): V {
       let value= if (vec_map::contains(self, key)) {
            *vec_map::get(self, key)
        } else {
            default
        };
        value
    }
}