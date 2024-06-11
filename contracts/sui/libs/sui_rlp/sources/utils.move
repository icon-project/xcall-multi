module sui_rlp::utils {
     use std::vector::{Self};
     use std::string::{Self,String};
     public fun to_bytes_u32(number: u32): vector<u8> {
        let  mut bytes: vector<u8> = vector::empty();
        let mut i:u8=0;
        while(i < 4){
            let val =( (number>>(i * 8) & 0xFF) as u8) ;
            vector::push_back(&mut bytes,val);
            i=i+1;
        };
        bytes.reverse();
        bytes
    }

    // Convert bytes to u32
    public fun from_bytes_u32(bytes: &vector<u8>): u32 {let mut result = 0;
        let mut multiplier = 1;
        let length = vector::length(bytes);

        let mut i = length;
        while (i > 0) {
            i = i - 1;
            //std::debug::print(vector::borrow(bytes, i));
            result = result + ((*vector::borrow(bytes, i) as u32) * multiplier);
            //std::debug::print(&result);

            if (i > 0) {
            multiplier = multiplier * 256
            };
            
        };
        result
    }

     public fun to_bytes_u64(number: u64): vector<u8> {
        let  mut bytes: vector<u8> = vector::empty();
        let mut i:u8=0;
        while(i < 8){
            let val =( (number>>(i * 8) & 0xFF) as u8) ;
             vector::push_back(&mut bytes,val);
            i=i+1;
        };
        bytes.reverse();
        let mut prefix = vector<u8>[0];
        prefix.append(truncate_zeros(&bytes));
        prefix
    }

    fun truncate_zeros(bytes: &vector<u8>): vector<u8> {
        let mut i = 0;
        let mut started = false;
        let mut result: vector<u8> = vector::empty();
        while (i < vector::length(bytes)) {
            let val = *vector::borrow(bytes, i) as u8;
            if (val > 0 || started) {
                started = true;
                vector::push_back(&mut result, val);
            };

            i = i + 1;
        };

        result
    }

    // Convert bytes to u64
    public fun from_bytes_u64(bytes: &vector<u8>): u64 {
        let bytes = truncate_zeros(bytes);
        let mut result = 0;
        let mut multiplier = 1;
        let length = vector::length(&bytes);

        let mut i = length;
        while (i > 0) {
            i = i - 1;
            //std::debug::print(vector::borrow(bytes, i));
            result = result + ((*vector::borrow(&bytes, i) as u64) * (multiplier));
            //std::debug::print(&result);
            if (i > 0) {
            multiplier = multiplier * 256
            };
            
        };
        result
    }

    

    // Convert u128 to bytes
    public fun to_bytes_u128(number: u128): vector<u8> {
        let  mut bytes: vector<u8> = vector::empty();
        let mut i:u8=0;
        while(i < 16){
            let val = ((number>>(i * 8) & 0xFF) as u8) ;
             vector::push_back(&mut bytes,val);
            i=i+1;
        };
        bytes.reverse();
        let mut prefix = vector<u8>[0];
        prefix.append(truncate_zeros(&bytes));
        prefix
    }

    // Convert bytes to u128
    public fun from_bytes_u128(bytes: &vector<u8>): u128 {
        let bytes = truncate_zeros(bytes);
       let mut result = 0;
        let mut multiplier = 1;
        let length = vector::length(&bytes);

        let mut i = length;
        while (i > 0) {
            i = i - 1;
            //std::debug::print(vector::borrow(bytes, i));
            result = result + ((*vector::borrow(&bytes, i) as u128) * multiplier);
            //std::debug::print(&result);

            if (i > 0) {
            multiplier = multiplier * 256
            };
            
        };
        result
    }

    /* end is exclusive in slice*/
   public fun slice_vector(vec: &vector<u8>, start: u64, length: u64): vector<u8> {
        let mut result = vector::empty<u8>();
        let mut i = 0;
        while (i < length) {
            let value = *vector::borrow(vec, start + i);
            vector::push_back(&mut result, value);
            i = i + 1;
        };
        //std::debug::print(&result);
        result
    }

   
}

module sui_rlp::utils_test {
    use sui_rlp::utils::{Self};
     use std::vector::{Self};
     use std::debug;
     use sui::bcs::{Self};


      #[test]
    fun test_u32_conversion() {
        let num= (122 as u32);
        let bytes= utils::to_bytes_u32(num);
        let converted=utils::from_bytes_u32(&bytes);
        assert!(num==converted,0x01);
        
    }

    #[test]
    fun test_u64_conversion() {
        let num= (55000 as u64);
        let bytes= utils::to_bytes_u64(num);
        let converted=utils::from_bytes_u64(&bytes);
        std::debug::print(&bytes);
        std::debug::print(&converted);
        assert!(num==converted,0x01);
        
    }

    #[test]
    fun test_u128_conversion() {
        let num= (1222223333 as u128);
        let bytes= utils::to_bytes_u128(num);
        std::debug::print(&bytes);
        let converted=utils::from_bytes_u128(&bytes);
        std::debug::print(&converted);
        assert!(num==converted,0x01); 
        
    }

    #[test]
    fun test_vector_slice() {
        let data=create_vector(10);
        let slice= utils::slice_vector(&data,0,3);
        let expected= create_vector(3);
       //debug::print(&expected);
       //debug::print(&slice);
       //debug::print(&data);
        assert!(slice==expected,0x01);

        
    }

    fun create_vector(n:u8):vector<u8>{
         let mut data=vector::empty();
        let mut i=0;
        while(i < n){
            vector::push_back(&mut data,i);
            i=i+1;
        };
        data
    }


}