module sui_rlp::decoder {
    use sui_rlp::utils::{Self};
     use std::vector::{Self};
     use sui::bcs;
     use std::string::{Self,String};
     use std::debug;
     const InvalidPrefix:u64=1;


    public fun decode(encoded: &vector<u8>): vector<u8> {
            assert!(vector::length(encoded) > 0, 0x1);
            let byte = *vector::borrow(encoded, 0);
            if (byte == 0x80) {
                vector::empty()
            } else if (byte < 0x80) {
                vector::singleton(byte)
            } else if (byte < 0xb8) {
                let length = byte - 0x80;
                utils::slice_vector(encoded, 1, length as u64)
            } else {
                let length_len = byte - 0xb7;
                let length_bytes = utils::slice_vector(encoded, 1, length_len as u64);
               //debug::print(&length_bytes);
                let length = utils::from_bytes_u64(&length_bytes);
                let data_start = (length_len + 1) as u64;
                utils::slice_vector(encoded, data_start, length)
            }
    }

     public fun decode_length(data:&vector<u8>,offset:u8):u64{
       let length=vector::length(data);
       let len= if (length==0){
            0
        }else if(length <56){
           ((*vector::borrow(data,0)-offset) as u64)
           


        }else {
            let length_len=*vector::borrow(data,0)-offset;
          
            let length_bytes=utils::slice_vector(data,1,(length_len as u64));
            utils::from_bytes_u64(&length_bytes)

        };

        len
  
    }

     public fun decode_list(list: &vector<u8>): vector<vector<u8>> {

        let mut values: vector<vector<u8>> = vector::empty();

        if(vector::length(list)>0){
        let prefix = *vector::borrow(list, 0);
        let list_length = if (prefix <= 0xf7) {
            (prefix - 0xc0) as u64
        } else {
            let length_of_length = (prefix - 0xf7) as u64;
            utils::from_bytes_u64(&utils::slice_vector(list, 1, length_of_length))
        };
        let start=vector::length(list)-list_length;
        let encoded= utils::slice_vector(list,start,vector::length(list)-start);
        let mut i: u64 = 0;
        while (i < vector::length(&encoded)) {
            let prefix = *vector::borrow(&encoded,i);
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

            }else if(prefix==0xc0){
               vector::push_back(&mut values,vector::empty<u8>());
               i=i+1;

            }else if(prefix > 0xc0 && prefix < 0xf7){
                let length=((prefix-0xc0) as u64);
                vector::push_back(&mut values,utils::slice_vector(&encoded, ((i) as u64), length+1));
                i = i+(length+1);
            
            } else if(prefix >0xb7 && prefix < 0xc0){
                let length_length = ((prefix - 0xB7) as u64);
                let length = utils::from_bytes_u64(&utils::slice_vector(&encoded, ((i + 1) as u64), length_length));
             
                vector::push_back(&mut values,utils::slice_vector(&encoded, ((i + length_length + 1) as u64), length));
                i = i+(length_length + length + 1);
            }else if(prefix > 0xf7){
                let length_length = ((prefix - 0xF7) as u64);
                let length = utils::from_bytes_u64(&utils::slice_vector(&encoded, ((i + 1) as u64), length_length));
                if(prefix == 0xf8 && length==0){
                    values.push_back(vector::empty());
                }else{
                    vector::push_back(&mut values,utils::slice_vector(&encoded, ((i ) as u64),length + length_length + 1));
                };
                i = i+(length_length + length + 1);

            } else {
                abort InvalidPrefix
            };
        };
        };
        values
    }

     public fun decode_u8(vec:&vector<u8>):u8{
       
        *vector::borrow(vec,0)

    }

    public fun decode_u64(vec:&vector<u8>):u64{
         let num =utils::from_bytes_u64(vec);
         num
        
    }

    public fun decode_u128(vec:&vector<u8>):u128{
         
           let num =utils::from_bytes_u128(vec);
         num
    }

    public fun decode_string(vec:&vector<u8>):String{
         string::utf8(*vec)
    }

    public fun decode_strings(vec:&vector<u8>):vector<String>{
        
        let vecs=decode_list(vec);
        
        let mut strings=vector::empty<String>();
        let mut i=0;
        while(i < vector::length(&vecs)){
           let item= vector::borrow(&vecs,i);
           vector::push_back(&mut strings,decode_string(item));
           i=i+1;
           
        };
        strings
    }

    public fun decode_address(vec:&vector<u8>):address{
         let mut bcs = bcs::new(*vec);
         bcs::peel_address(&mut bcs)
    }

 
}