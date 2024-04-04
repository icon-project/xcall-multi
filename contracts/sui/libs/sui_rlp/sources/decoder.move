module sui_rlp::decoder {
    use sui_rlp::utils::{Self};
     use std::vector::{Self};
     use sui::bcs;
     use std::string::{Self,String};
     use std::debug;
     #[test_only] friend sui_rlp::rlp_tests;

     public fun decode(encoded:&vector<u8>):vector<u8>{
        assert(vector::length(encoded) > 0, 0x1);
        let byte = *vector::borrow(encoded,0);
       let decoded= if (byte==0x80) {
           vector::empty()
       } else if (byte < 0x80) {
            vector::singleton(byte)
        } else if (byte < 0xb8) {
            let length = byte - 0x80;
            let data = utils::slice_vector(encoded,1, ((length) as u64));
            data
        } else {
            let length_len = byte - 0xb7;
            debug::print(&length_len);
            let length_bytes= utils::slice_vector(encoded,1,((length_len) as u64));
            let length = utils::from_bytes_u64(&length_bytes);
            debug::print(&length);
            let data_start = ((length_len + 1) as u64);
            debug::print(&data_start);
            let data = utils::slice_vector(encoded,data_start, length);
            data
        };
        decoded
     }


      public fun decode_length(encoded: &vector<u8>): u64 {
        let length=vector::length(encoded);
        let len= if (length == 0) {
            0
        }else if (length == 1) {
            let len= (*vector::borrow(encoded,0) as u64);
            len
        } else {
            let byte=*vector::borrow(encoded,0);
            let length_len = byte - 0xb7;
            let  decoded_length: u64 = 0;
            let i=1;
            while(i < length_len){
                let byte=*vector::borrow(encoded,(i as u64));
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
            debug::print(&prefix);
            if (prefix==0x80){
                vector::push_back(&mut values,vector::empty());
                i = i+1;
            }else if (prefix < 0x80) {
                vector::push_back(&mut values,vector::singleton(prefix));
                i = i+1;
            } else if( prefix > 0x80 && prefix < 0xB8) {
                let length = ((prefix - 0x80) as u64);
                vector::push_back(&mut values,utils::slice_vector(&encoded, ((i + 1) as u64), length));
                i = i+(length + 1);
            } else {
                let length_length = ((prefix - 0xB7) as u64);
                let length = decode_length(&utils::slice_vector(&encoded, ((i + 1) as u64), length_length));
                vector::push_back(&mut values,utils::slice_vector(&encoded, ((i + length_length + 1) as u64), length));
                i = i+(length_length + length + 1);
            };
        };
        values
    }

     public fun decode_u8(vec:&vector<u8>):u8{
        let decoded=decode(vec);
        *vector::borrow(&decoded,0)

    }

    public fun decode_u64(vec:&vector<u8>):u64{
         let decoded=decode(vec);
         let num =utils::from_bytes_u64(&decoded);
         num
        
    }

    public fun decode_u128(vec:&vector<u8>):u128{
         let decoded=decode(vec);
           let num =utils::from_bytes_u128(&decoded);
         num
    }

    public fun decode_string(vec:&vector<u8>):String{
         let decoded=decode(vec);
         string::utf8(decoded)
    }

    public fun decode_address(vec:&vector<u8>):address{
         let decoded=decode(vec);
         let bcs = bcs::new(decoded);
         bcs::peel_address(&mut bcs)
    }

 
}