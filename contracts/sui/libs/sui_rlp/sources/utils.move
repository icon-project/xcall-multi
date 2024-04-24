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
        bytes
    }

    // Convert bytes to u32
    public fun from_bytes_u32(bytes: &vector<u8>): u32 {
        assert(vector::length(bytes) == 4, 0x1); // Ensure bytes length is correct
        let  mut number: u32 = 0;
       let  mut i = 0;

        while (i < 4) {
        let num = ((*vector::borrow(bytes,i)) as u32);
        number =number | num << ((i * 8) as u8);
        i =i+ 1;
        };
        number
    }

     public fun to_bytes_u64(number: u64): vector<u8> {
        let  mut bytes: vector<u8> = vector::empty();
        let mut i:u8=0;
        while(i < 8){
            let val =( (number>>(i * 8) & 0xFF) as u8) ;
            if (val > 0) {
             vector::push_back(&mut bytes,val);
            };
            i=i+1;
        };
        bytes
    }

    // Convert bytes to u64
    public fun from_bytes_u64(bytes: &vector<u8>): u64 {
        let  mut number: u64 = 0;
       let  mut i = 0;

        while (i < vector::length(bytes)) {
        let num = ((*vector::borrow(bytes,i)) as u64);
        number =number | num << ((i * 8) as u8);
        i =i+ 1;
        };
        number
    }

    // Convert u128 to bytes
    public fun to_bytes_u128(number: u128): vector<u8> {
        let  mut bytes: vector<u8> = vector::empty();
        let mut i:u8=0;
        while(i < 16){
            let val = ((number>>(i * 8) & 0xFF) as u8) ;
            if (val > 0) {
             vector::push_back(&mut bytes,val);
            };
            
            i=i+1;
        };
        bytes
    }

    // Convert bytes to u128
    public fun from_bytes_u128(bytes: &vector<u8>): u128 {
       let  mut number: u128 = 0;
       let mut i:u64 = 0;

        while (i < vector::length(bytes)) {
        number =number | (*vector::borrow(bytes,i) as u128) << (i * 8 as u8) ;
        i =i+ 1;
        };
        number
    }
    /* end is exclusive in slice*/
    public fun slice_vector(arr: &vector<u8>, start: u64, length: u64): vector<u8> {
        let  mut sliced: vector<u8> = vector::empty();
        let mut start=start;
        let end = start + length;
        while(start < end){
            let item=*vector::borrow(arr,start);
            vector::push_back(&mut sliced, item);
            start=start+1;
        };
        sliced
    }
   
}

module sui_rlp::utils_test {
    use sui_rlp::utils::{Self};
     use std::vector::{Self};
     use std::debug;


      #[test]
    fun test_u32_conversion() {
        let num= (122 as u32);
        let bytes= utils::to_bytes_u32(num);
        let converted=utils::from_bytes_u32(&bytes);
        assert!(num==converted,0x01);
        
    }

    #[test]
    fun test_u64_conversion() {
        let num= (1222233 as u64);
        let bytes= utils::to_bytes_u64(num);
        let converted=utils::from_bytes_u64(&bytes);
        assert!(num==converted,0x01);
        
    }

    #[test]
    fun test_u128_conversion() {
        let num= (1222223333 as u128);
        let bytes= utils::to_bytes_u128(num);
        let converted=utils::from_bytes_u128(&bytes);
        assert!(num==converted,0x01); 
        
    }

    #[test]
    fun test_vector_slice() {
        let data=create_vector(10);
        let slice= utils::slice_vector(&data,0,3);
        let expected= create_vector(3);
        debug::print(&expected);
        debug::print(&slice);
        debug::print(&data);
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