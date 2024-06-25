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
        }else if(len <= 55){
           let mut result=encode_length(len,0x80);
           vector::append(&mut result,*bytes);
           result
        }else{
            let mut result=encode_length(len,0xb7);
            vector::append(&mut result,*bytes);
           result
        };
        //std::debug::print(&encoded);
        encoded
        
    }

    public fun encode_list(list:&vector<vector<u8>>,raw:bool):vector<u8>{
        //std::debug::print(&b"ENCODELIST".to_string());
        //std::debug::print(list);
        let mut result=vector::empty();
        let mut encoded_list = vector::empty<u8>();
        let mut list=*list;
        if(vector::length(&list)>0){
            vector::reverse(&mut list);

        while(!vector::is_empty(&list)){
            if(raw==true){
              vector::append(&mut result,encode(&vector::pop_back(&mut list)));
            }else{
              vector::append(&mut result,vector::pop_back(&mut list));
              //std::debug::print(&result);
            };
           
        };

        let total_length = result.length();
        let len=vector::length(&result);

        if( total_length<= 55){
            encoded_list=encode_length(len,0xc0);
            vector::append(&mut encoded_list,result);

        } else {
           let length_bytes = utils::to_bytes_u64(len);
           let prefix = (0xf7 + vector::length(&length_bytes)) as u8;
            //std::debug::print(&b"PREFIX".to_string());
            //std::debug::print(&prefix);
           vector::push_back(&mut encoded_list, prefix);
            //std::debug::print(&encoded_list);
           vector::append(&mut encoded_list, length_bytes);
                       //std::debug::print(&encoded_list);

           vector::append(&mut encoded_list, result);
                       //std::debug::print(&encoded_list);


        }

        }else{
            vector::push_back(&mut encoded_list,0xc0);

        };
        //std::debug::print(&b"FINAL_ENCODED_LIST".to_string());
        //std::debug::print(&encoded_list);
        encoded_list   
    }

    public fun encode_length(len:u64,offset:u8):vector<u8>{
        let mut length_info=vector::empty<u8>();
        if (len < 56) {
            let len_u8=(len as u8);
            vector::push_back(&mut length_info,(offset+len_u8));
        }else {
        let length_bytes=utils::to_bytes_u64(len);
        let length_byte_len=vector::length(&length_bytes);
        let length_byte_len=offset+(length_byte_len as u8);
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



    
