#[test_only]
module sui_rlp::rlp_tests {
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::string::{Self};

     #[test]
    fun test_encode_string() {
        let bytes=b"hello world! a very long string value";
        let val=string::utf8(bytes);
        let encoded= encoder::encode_string(&val);
        let decoded_val= decoder::decode(&encoded);
        let decoded=decoder::decode_string(&decoded_val);
        assert!(decoded==val,0x01);
    }


    #[test]
    fun test_encode_long_string() {
        let bytes=b"Lorem Ipsum is simply dummy text of the printing and typesetting industry.Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum";
        let val=string::utf8(bytes);
        let encoded= encoder::encode_string(&val);
        let decoded_val= decoder::decode(&encoded);
        let decoded= decoder::decode_string(&decoded_val);
        assert!(decoded==val,0x01);
    }


    #[test]
    fun test_encode_u8() {
    
        let val=(13 as u8);
        let encoded= encoder::encode_u8(val);
        let decoded_val= decoder::decode(&encoded);
        let decoded= decoder::decode_u8(&decoded_val);
        assert!(decoded==val,0x01);
    }

    #[test]
    fun test_encode_u64() {    
       
        let val=(18446744073709551615 as u64);
        let encoded= encoder::encode_u64(val);
        std::debug::print(&encoded);
        let decoded_val= decoder::decode(&encoded);
        let decoded= decoder::decode_u64(&decoded_val);
        std::debug::print(&decoded);
        assert!(decoded==val,0x01);
    }

    #[test]
    fun test_encode_u128() {
        let val=(2*100000000000000000 as u128);
        let encoded= encoder::encode_u128(val);
        std::debug::print(&encoded);
        let decoded_val= decoder::decode(&encoded);
        let decoded= decoder::decode_u128(&decoded_val);
        std::debug::print(&decoded);
        assert!(decoded==val,0x01);
    }

       #[test]
    fun max_test_encode_u128() {
        let val: u128=340282366920938463463374607431768211455;
        let encoded= encoder::encode_u128(val);
        std::debug::print(&encoded);
        let decoded_val= decoder::decode(&encoded);
        let decoded= decoder::decode_u128(&decoded_val);
        std::debug::print(&decoded);
        assert!(decoded==val,0x01);
    }

    fun create_list():vector<vector<u8>>{
        let mut list=vector::empty();
        vector::push_back(&mut list, encoder::encode_u8(44));
        vector::push_back(&mut list, encoder::encode_u64(444444));
        vector::push_back(&mut list, encoder::encode_u128(444444444444));
        vector::push_back(&mut list, encoder::encode_string(&string::utf8(b"hello")));
        vector::push_back(&mut list, encoder::encode_list(&vector::empty(),false));
        list
    }


     #[test]
    fun test_encoding_u128(){
        let num:u128=100;
        let bytes=x"64";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);
        
        ////
        /// 
        let num:u128=200;
        let bytes=x"8200c8";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);
        ///
        /// 
        let num:u128=3000000;
        let bytes=x"832dc6c0";
        let encoded= encoder::encode_u128(num);
        std::debug::print(&encoded);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);

        let num:u128=273468273;
        let bytes=x"84104ccb71";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);

        let num:u128=2342312;
        let bytes=x"8323bda8";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);

        let num:u128=1233;
        let bytes=x"8204d1";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);

        let num:u128=412926;
        let bytes=x"83064cfe";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);

        let num:u128=9434628989898;
        let bytes=x"860894abb5a3ca";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);

        let num:u128=92625222222121112;
        let bytes=x"88014912261bca8898";
        let encoded= encoder::encode_u128(num);
        assert!(encoded==bytes);
        let decoded=decoder::decode_u128(&decoder::decode(&encoded));
        assert!(decoded==num);


       
    }

     



}