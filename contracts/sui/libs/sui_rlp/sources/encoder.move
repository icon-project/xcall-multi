module sui_rlp::encoder {
    use sui_rlp::utils::{Self};
    use std::vector::{Self};
    use std::string::{Self,String};
    use std::bcs;
    use std::debug;
    
    public fun encode(bytes:&vector<u8>):vector<u8> {
       
        let len=vector::length(bytes);
        let encoded= if(len==0){
            vector::singleton(0x80)
        } else if (len ==1 && *vector::borrow(bytes,0)<128){
            *bytes
        }else {
           let mut result=encode_length(len,0x80);
           vector::append(&mut result,*bytes);
           result
        };
        encoded
        
    }

    public fun encode_list(list:&vector<vector<u8>>,raw:bool):vector<u8>{
        let mut result=vector::empty();
        let mut list=*list;
        if(vector::length(&list)>0){
            vector::reverse(&mut list);

        while(!vector::is_empty(&list)){
            if(raw==true){
              vector::append(&mut result,encode(&vector::pop_back(&mut list)));
            }else{
              vector::append(&mut result,vector::pop_back(&mut list));
            };
           
        };
         let len=vector::length(&result);
         let mut length_buff=encode_length(len,192);
         vector::append(&mut length_buff,result);
         result=length_buff;
         

        }else{
            vector::push_back(&mut result,0xc0);

        };
       
        result
        
    }

    public fun encode_length(len:u64,offset:u8):vector<u8>{
        let mut length_info=vector::empty<u8>();
        if (len < 56) {
            let len_u8=(len as u8);
            vector::push_back(&mut length_info,(offset+len_u8));
        }else {
        let length_bytes=utils::to_bytes_u64(len);
        let length_byte_len=vector::length(&length_bytes);
        let length_byte_len=offset+55+(length_byte_len as u8);
        vector::push_back(&mut length_info,length_byte_len);
        vector::append(&mut length_info,length_bytes);
        
        };
        length_info
       
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

    public fun encode_strings(str:&vector<String>):vector<u8>{
        let mut vec=vector::empty<vector<u8>>();
        let mut i=0;
        let l= vector::length(str);
        while(i < l){
             let item=*vector::borrow(str,i);
             vector::push_back(&mut vec,encode_string(&item));
            i=i+1;
        };
        encode_list(&vec,false)

    }

    public fun encode_address(addr:&address):vector<u8> {
        let vec= bcs::to_bytes(addr);
        encode(&vec)
    }

   




}