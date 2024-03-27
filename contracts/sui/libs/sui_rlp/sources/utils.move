module sui_rlp::utils {
     use std::vector::{Self};

     public fun to_bytes_u64(number: u64): vector<u8> {
        let  bytes: vector<u8> = vector::empty();
        let i:u8=0;
        while(i < 8){
            let val =( (number>>(i * 8) & 0xFF) as u8) ;
            vector::push_back(&mut bytes,val);
            i=i+1;
        };
        bytes
    }

    // Convert bytes to u64
    public fun from_bytes_u64(bytes: &vector<u8>): u64 {
        assert(vector::length(bytes) == 8, 0x1); // Ensure bytes length is correct
        let  number: u64 = 0;
       let  i = 0;

        while (i < 8) {
        let num = ((*vector::borrow(bytes,i)) as u64);
        number =number | num << ((i * 8) as u8);
        i =i+ 1;
        };
        number
    }

    // Convert u128 to bytes
    public fun to_bytes_u128(number: u128): vector<u8> {
        let  bytes: vector<u8> = vector::empty();
        let i:u8=0;
        while(i < 16){
            let val = ((number>>(i * 8) & 0xFF) as u8) ;
            vector::push_back(&mut bytes,val);
            i=i+1;
        };
        bytes
    }

    // Convert bytes to u128
    public fun from_bytes_u128(bytes: &vector<u8>): u128 {
       assert(vector::length(bytes) == 16, 0x1); // Ensure bytes length is correct
       let  number: u128 = 0;
       let  i:u64 = 0;

        while (i < 16) {
        number =number | (*vector::borrow(bytes,i) as u128) << (i * 8 as u8) ;
        i =i+ 1;
        };
        number
    }

    public fun slice_vector(arr: vector<u8>, start: u64, length: u64): vector<u8> {
        let  sliced: vector<u8> = vector::empty();
        let end = start + length;
        while(start < end){
            let item=*vector::borrow(&arr,start);
            vector::push_back(&mut sliced, item);
            start=start+1;
        };
        sliced
    }
   #[test_only] friend sui_rlp::utils_test;
}

module sui_rlp::utils_test {
    use sui_rlp::utils::{Self};

    #[test]
    fun test_u64_conversion() {
        let num= (122 as u64);
        let bytes= utils::to_bytes_u64(num);
        let converted=utils::from_bytes_u64(&bytes);
        assert!(num==converted,0x01);
        
    }

    #[test]
    fun test_u128_conversion() {
        let num= (122 as u128);
        let bytes= utils::to_bytes_u128(num);
        let converted=utils::from_bytes_u128(&bytes);
        assert!(num==converted,0x01); 
        
    }


}