#[test_only]
module sui_rlp::rlp_tests {
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::string::{Self};
    use std::debug;

     #[test]
    fun test_encode_string() {
        let bytes=b"hello world! a very long string value";
        let val=string::utf8(bytes);
        let encoded= encoder::encode_string(&val);
        let decoded= decoder::decode_string(&encoded);
        assert!(decoded==val,0x01);
    }


    #[test]
    fun test_encode_long_string() {
        let bytes=b"Lorem Ipsum is simply dummy text of the printing and typesetting industry.Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum";
        let val=string::utf8(bytes);
        let encoded= encoder::encode_string(&val);
        let decoded= decoder::decode_string(&encoded);
        assert!(decoded==val,0x01);
    }


    #[test]
    fun test_encode_u8() {
    
        let val=(13 as u8);
        let encoded= encoder::encode_u8(val);
        let decoded= decoder::decode_u8(&encoded);
        assert!(decoded==val,0x01);
    }

    #[test]
    fun test_encode_u64() {
       
        let val=(12345667 as u64);
        let encoded= encoder::encode_u64(val);
        let decoded= decoder::decode_u64(&encoded);
        assert!(decoded==val,0x01);
    }

    #[test]
    fun test_encode_u128() {
        let val=(2245667788 as u128);
        let encoded= encoder::encode_u128(val);
        let decoded= decoder::decode_u128(&encoded);
        assert!(decoded==val,0x01);
    }

}