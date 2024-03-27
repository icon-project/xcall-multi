module sui_rlp::decode {
    use sui_rlp::utils::{Self};
     use std::vector::{Self};


      public fun decode_length(encoded: vector<u8>): u64 {
        let length=vector::length(&encoded);
        let len= if (length == 0) {
            0
        }else if (length == 1) {
            let len= (*vector::borrow(&encoded,0) as u64);
            len
        } else {
            let byte=*vector::borrow(&encoded,0);
            let length_len = byte - 0xb7;
            let  decoded_length: u64 = 0;
            let i=1;
            while(i < length_len){
                let byte=*vector::borrow(&encoded,(i as u64));
                decoded_length = (decoded_length << 8) | (byte as u64);
                i=i+1;

            };
         decoded_length
        };
        len
    }


     public fun decode_list(encoded: vector<u8>): vector<vector<u8>> {
        let values: vector<vector<u8>> = vector::empty();
        let i: u64 = 0;
        while (i < vector::length(&encoded)) {
            let prefix = *vector::borrow(&encoded,i);
            if (prefix < 0x80) {
                vector::push_back(&mut values,vector::singleton(prefix));
                i = i+1;
            } else if( prefix < 0xB8) {
                let length = ((prefix - 0x80) as u64);
                vector::push_back(&mut values,utils::slice_vector(encoded, ((i + 1) as u64), length));
                i = i+(length + 1);
            } else {
                let length_length = ((prefix - 0xB7) as u64);
                let length = decode_length(utils::slice_vector(encoded, ((i + 1) as u64), length_length));
                vector::push_back(&mut values,utils::slice_vector(encoded, ((i + length_length + 1) as u64), length));
                i = i+(length_length + length + 1);
            };
        };
        values
    }

 
}