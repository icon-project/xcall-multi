module sui_rlp::utils {
    use std::bcs;


    // Convert bytes to u32
    public fun from_bytes_u32(bytes: &vector<u8>): u32 {
        let mut bytes= truncate_zeros(bytes);
        bytes.reverse();
        let mut diff= 4-bytes.length();
        while (diff > 0) {
            bytes.push_back(0_u8);
            diff=diff-1;
        };
        sui::bcs::peel_u32(&mut sui::bcs::new(bytes))
    }


    // Convert bytes to u64
    public fun from_bytes_u64(bytes: &vector<u8>): u64 {
        let mut bytes= truncate_zeros(bytes);
        bytes.reverse();
        let mut diff= 8-bytes.length();
        while (diff > 0) {
            bytes.push_back(0_u8);
            diff=diff-1;
        };
        sui::bcs::peel_u64(&mut sui::bcs::new(bytes))

    }

    // Convert bytes to u128
    public fun from_bytes_u128(bytes: &vector<u8>): u128 {
        let mut bytes= truncate_zeros(bytes);
        bytes.reverse();
        let mut diff= 16-bytes.length();
        while (diff > 0) {
            bytes.push_back(0_u8);
            diff=diff-1;
        };
        sui::bcs::peel_u128(&mut sui::bcs::new(bytes))

    }

    public fun to_bytes_u128(number:u128,signed:bool):vector<u8>{
        let bytes=bcs::to_bytes(&number);
        to_signed_bytes(bytes,signed)
    }


    public fun to_bytes_u64(number:u64,signed:bool):vector<u8>{
        let bytes=bcs::to_bytes(&number);
        to_signed_bytes(bytes,signed)
    }

    public fun to_bytes_u32(number: u32,signed:bool): vector<u8> {
        let bytes=bcs::to_bytes(&number);
        to_signed_bytes(bytes,signed)
    }

    fun to_signed_bytes(mut bytes:vector<u8>,signed:bool):vector<u8>{
        bytes.reverse();
        let truncated=truncate_zeros(&bytes);
        if(signed==false){
            return truncated
        };
        let first_byte=*truncated.borrow(0);

        if (first_byte >= 128) {
            let mut prefix = vector<u8>[0];
            prefix.append(truncated);
            prefix

        }else {
            truncated
        }

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
        if (result.length()==0){
            vector<u8>[0]
        }else{
            result
        }
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
        result
    }


}

module sui_rlp::utils_test {
    use sui_rlp::utils::{Self};


    #[test]
    fun test_u32_conversion() {
        let num= (122 as u32);
        let bytes= utils::to_bytes_u32(num,true);
        let converted=utils::from_bytes_u32(&bytes);
        assert!(num==converted,0x01);

    }

    #[test]
    fun test_u64_conversion() {
        let num= (55000 as u64);
        let bytes= utils::to_bytes_u64(num,true);
        let converted=utils::from_bytes_u64(&bytes);
        std::debug::print(&bytes);
        std::debug::print(&converted);
        assert!(num==converted,0x01);

    }

    #[test]
    fun test_u128_conversion() {
        let num= (1222223333 as u128);
        let bytes= utils::to_bytes_u128(num,true);
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