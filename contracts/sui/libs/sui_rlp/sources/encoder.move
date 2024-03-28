module sui_rlp::encoder {
    use sui_rlp::utils::{Self};
    use std::vector::{Self};
    use std::string::{Self,String};
    use std::bcs;
    #[test_only] friend sui_rlp::rlp_tests;
    public fun encode(bytes:&vector<u8>):vector<u8> {
       
        let len=vector::length(bytes);
        let encoded= if (len ==1 && *vector::borrow(bytes,0)<128){
            *bytes
        }else {
           let result=vector::empty();
           encode_length(&mut result,len);
           vector::append(&mut result,*bytes);
           result
        };
        encoded
        
    }

    public fun encode_list(list:vector<vector<u8>>):vector<u8>{
        let result=vector::empty();
        vector::reverse(&mut list);
        while(!vector::is_empty(&list)){
            vector::append(&mut result,vector::pop_back(&mut list))
        };
        let list_length=vector::length(&result);
        let encoded=vector::empty();
        encode_length(&mut encoded,list_length);
        vector::append(&mut encoded,result);
        encoded
    }

    public fun encode_length(buff:&mut vector<u8>,len:u64){
        if (len < 56) {
            let len_u8=(len as u8);
            vector::push_back(buff,(128+len_u8));
        }else {
        let length_bytes=utils::to_bytes_u64(len);
        let length_byte_len=vector::length(&length_bytes);
        let length_byte_len=183+(length_byte_len as u8);
        vector::push_back(buff,length_byte_len);
        vector::append(buff,length_bytes);
        }
       
    }

    public fun encode_u8(num:u8):vector<u8>{
        let vec=vector::singleton(num);
        encode(&vec)

    }

    public fun encode_u64(num:u64):vector<u8>{
        let vec= utils::to_bytes_u64(num);
        encode(&vec)
        
    }

    public fun encode_u128(num:u128):vector<u8>{
        let vec= utils::to_bytes_u128(num);
        encode(&vec)
    }

    public fun encode_string(val:&String):vector<u8>{
        let vec= string::bytes(val);
        encode(vec)
    }

    public fun encode_address(addr:&address):vector<u8> {
        let vec= bcs::to_bytes(addr);
        encode(&vec)
    }

   




}