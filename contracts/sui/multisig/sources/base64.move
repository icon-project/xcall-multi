module multisig::base64 {
     use std::vector::{Self};
    use std::string::{Self,String};
    use sui::vec_map::{Self, VecMap};

    const BASE64_CHARS: vector<u8> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const PADDING_CHAR: vector<u8> = b"=";


public fun encode(input:&vector<u8>):String {
   
    let mut output:vector<u8> = vector::empty();
    let mut i=0;
    while (i <input.length()){
        let b1:u8= *input.borrow(i);
        let b2:u8= if(i+1 < input.length()){
              *input.borrow(i+1)
        }else {
            0
        };

        let b3:u8= if((i+2)<input.length()){
              *input.borrow(i+2)
        }else {
            0
        };

        let triple=((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
       output.push_back(*BASE64_CHARS.borrow(((triple >> 18) & 0x3F) as u64));
       output.push_back(*BASE64_CHARS.borrow(((triple >> 12) & 0x3F) as u64));

        if (i + 1 < input.length()) {
            output.push_back(*BASE64_CHARS.borrow(((triple >> 6) & 0x3F) as u64));
        } else {
            output.push_back(*PADDING_CHAR.borrow(0));
        };

        if (i + 2 < input.length()) {
            output.push_back(*BASE64_CHARS.borrow((triple & 0x3F) as u64));
        } else {
            output.push_back(*PADDING_CHAR.borrow(0));
        };

        i =i+ 3;
    };
    string::utf8(output)
}

public fun decode(input:&String):vector<u8>{
    let char_index=get_char_map();
    let mut output = vector::empty<u8>();
    let input_bytes = input.as_bytes();
    let mut i = 0;
    while( i < input_bytes.length()){
        let b1 = *char_index.get(input_bytes.borrow(i));
        let b2 = *char_index.get(input_bytes.borrow(i + 1));
        let b3 = if (i + 2 < input_bytes.length()) { 
                     let key=input_bytes.borrow(i+2);
                     let val:u32 = if (char_index.contains(key)) {
                        *char_index.get(key)
                        } else {
                            64
                        };
                    val
                } else { 
                    64 
                };
        let b4 = if (i + 3 < input_bytes.length()) {
            let key=input_bytes.borrow(i+3);
            let val:u32=   if (char_index.contains(key)) {
                *char_index.get(key)
                } else {
                    64
                };
            val
        } else { 

            64 
        };

        let triple = (b1 << 18) | (b2 << 12) | (b3 << 6) | b4;

        output.push_back(((triple >> 16) & 0xFF) as u8);

        if (b3 != 64) {
        output.push_back(((triple >> 8) & 0xFF) as u8);
        };
        if (b4 != 64) {
        output.push_back((triple & 0xFF) as u8);
        };

        i = i+4;

    };
   output

}



  fun get_char_map():VecMap<u8,u32>{
    let mut char_map = vec_map::empty<u8,u32>();
    let mut i:u64=0;
    while( i < BASE64_CHARS.length()){
        let c=*BASE64_CHARS.borrow(i);
        char_map.insert(c,(i as u32));
         i=i+1;
    };
    char_map
  }
}


#[test_only]
module multisig::base64_tests {
    use sui::hash::{Self};
    use multisig::base64::{Self};
    #[test]
    fun test_base64(){
       let input = b"Hello, World!";
        let encoded = base64::encode(&input);
        assert!(encoded.as_bytes()==b"SGVsbG8sIFdvcmxkIQ==");

         let decoded = base64::decode(&encoded);
         std::debug::print(&encoded);
        assert!(decoded==input);

    }


}



