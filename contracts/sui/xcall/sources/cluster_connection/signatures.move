#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]

module xcall::signatures {
    use sui::linked_table::{Self, LinkedTable};
    use sui::types as sui_types;
    use std::string::{Self, String};
    use sui::event;
    use sui::hash::{Self};
    use sui::vec_map::{Self, VecMap};
    use sui::table::{Table,Self};
    use sui::bcs::{Self};
    use sui::address::{Self};
    use sui::{ed25519::ed25519_verify};
    use sui::{ecdsa_k1::secp256k1_verify};
    use sui::{ecdsa_r1::secp256r1_verify};

    const BASE64_CHARS: vector<u8> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const PADDING_CHAR: vector<u8> = b"=";

    /** signature schemes*/
    const FlagED25519 :u8= 0x00;
    const FlagSecp256k1 :u8= 0x01;
    const FlagSecp256r1 :u8= 0x02;
	const FlagMultiSig :u8= 0x03;

    /* hash algorithm*/
    const KECCAK256: u8 = 0x00;
    const SHA256: u8 = 0x01;

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

  public(package) fun pubkey_to_sui_address(pubkey:&String):(vector<u8>,address){
    let pubkey_bytes = decode(pubkey);
    let sui_address = address::from_bytes(hash::blake2b256(&pubkey_bytes));
    (pubkey_bytes,sui_address)
  }

  public(package) fun get_pubkey_from_signature(raw_signature:&vector<u8>):vector<u8>{
    let (signature,mut pubkey,scheme)= split_signature(raw_signature);
    pubkey.insert(scheme, 0);
    pubkey
  }

    public(package) fun verify_signature(pubkey:&vector<u8>,raw_signature:&vector<u8>,msg:&vector<u8>):bool{
        let flag=*pubkey.borrow(0);
        let public_key=slice_vector(pubkey,1,pubkey.length()-1);
        let (signature,pub,_scheme)= split_signature(raw_signature);
        
        let intent_msg= get_intent_message(msg);
        let digest= hash::blake2b256(&intent_msg);

        let verify= if(flag==FlagED25519){

            ed25519_verify(&signature,&public_key,&digest)

        }else if(flag==FlagSecp256k1){
            
            secp256k1_verify(&signature,&public_key,msg,SHA256)

        }else if (flag==FlagSecp256r1){

            secp256r1_verify(&signature,&public_key,msg,SHA256)
        }else {
            return false
        };
        verify
    }

    fun get_intent_message(msg:&vector<u8>):vector<u8>{
        let mut intent_message:vector<u8> =vector::empty();
        intent_message.push_back(0x00);
        intent_message.push_back(0x00);
        intent_message.push_back(0x00);
        intent_message.append(*msg);
        intent_message
    }

    fun split_signature(raw_signature:&vector<u8>):(vector<u8>,vector<u8>,u8){
        let scheme=*raw_signature.borrow(0);
        let length= if (scheme==FlagED25519){32}else{33};
        let signature=slice_vector(raw_signature,1,64);
        let pubkey=slice_vector(raw_signature,raw_signature.length()-length,length);
        return (signature,pubkey,scheme)
    }

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

#[test_only]
module xcall::signature_tests {
    use sui::hash::{Self};
    use xcall::signatures::{decode, pubkey_to_sui_address, get_pubkey_from_signature, verify_signature};
    use std::debug::print;
    use sui::{ed25519::ed25519_verify};

    #[test]
    fun test_base64(){
       let input = b"SGVsbG8sIFdvcmxkIQ==".to_string();
       let decoded = decode(&input);
       assert!(decoded==b"Hello, World!",0x01);
    }

    #[test]
    fun test_pubkey_to_sui_address(){
        let input = b"AL0hUNIiz5Q2fv0siZc75ce3aOyUpiiI+Q8Rmfay4K/X".to_string();
        let (_pubkey_bytes,sui_address) = pubkey_to_sui_address(&input);
        assert!(sui_address==@0xdefd91e5bfdb22edc6cabe1e9490549377047ec476354241910685876f1af34a,0x01);
    }

    #[test]
    fun test_get_pubkey_from_signature(){
        let input = b"ADQTOEZPWpPeZc9auXhwyciH7Pw6ny8xSxR+JVnSBnNktKAgTgxJ2EwHcErw55enK4w6c1uYaUv2gfY8Vb3X+Q6UHzNBcHIzSfMdKerK6XOCCmHbsI9Tpa8lGFvkm+Gbwg==".to_string();
        let pubkey = get_pubkey_from_signature(&decode(&input));
        assert!(pubkey==decode(&b"AJQfM0FwcjNJ8x0p6srpc4IKYduwj1OlryUYW+Sb4ZvC".to_string()),0x01);
    }

    #[test]
    fun test_verify_signature(){
        let data=x"6162636465666768";
        let pubkey = b"ALnG7hYw7z5xEUSmSNsGu7IoT3J0z77lP/zuUDzBpJIA".to_string();
        let signature = b"ALsKe6SiQqSYjIILlKjfmzEunnz0+DArU+4uBG522obq6cFSlqQhuN3bKcr6jVBSPgsEMAIW45PUXAc5oOq45gy5xu4WMO8+cRFEpkjbBruyKE9ydM++5T/87lA8waSSAA==".to_string();
        let verify = verify_signature(&decode(&pubkey),&decode(&signature),&data);
        assert!(verify,0x01);

        let data=x"6162636465666768";
        let pubkey = x"b9c6ee1630ef3e711144a648db06bbb2284f7274cfbee53ffcee503cc1a49200";
        let signature = x"c2ac48f07c55a8da3a0a762032a4956b08c63eebbe3f7a3f4cc56cd7fa23b1a3f7dec39e8e112162797d7c98f87d07314abff3cf594e42c27e942d3af43dcc07";


        let verify = ed25519_verify(&signature, &pubkey, &data);
        assert!(verify,0x01);

    }    
}
