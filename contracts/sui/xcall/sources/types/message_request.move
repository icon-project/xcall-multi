#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::message_request {
use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
    use std::vector::{Self};
     use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;



  public struct CSMessageRequest has store,drop,copy{
    from:NetworkAddress,
    to: String,
    sn:u128,
    message_type:u8,
    data:vector<u8>,
    protocols:vector<String>,
   }


    public fun create(from:NetworkAddress,
    to: String,
    sn:u128,
    message_type:u8,
    data:vector<u8>,
    protocols:vector<String>):CSMessageRequest {
        CSMessageRequest {
            from:from,
            to:to,
            sn:sn,
            message_type:message_type,
            data:data,
            protocols:protocols
        }
    }

    public fun new():CSMessageRequest {
        CSMessageRequest {
            from:network_address::from_string(string::utf8(b"")),
            to:string::utf8(b""),
            sn:0,
            message_type:0,
            data:vector::empty<u8>(),
            protocols:vector::empty<String>()
        }
    }

    public fun encode(req:&CSMessageRequest):vector<u8>{
          let mut list=vector::empty<vector<u8>>();
           vector::push_back(&mut list,network_address::encode(&req.from));
          vector::push_back(&mut list,encoder::encode_string(&req.to));
          vector::push_back(&mut list,encoder::encode_u128(req.sn));
          vector::push_back(&mut list,encoder::encode_u8(req.message_type));
          vector::push_back(&mut list,encoder::encode(&req.data));
          vector::push_back(&mut list,encoder::encode_strings(&req.protocols));

          let encoded=encoder::encode_list(&list,false);
          encoded
    }

    public fun decode(bytes:&vector<u8>):CSMessageRequest {
         let decoded=decoder::decode_list(bytes);
         let from= network_address::decode(vector::borrow(&decoded,0));
         let to= decoder::decode_string(vector::borrow(&decoded,1));
         let sn= decoder::decode_u128(vector::borrow(&decoded,2));
         let message_type= decoder::decode_u8(vector::borrow(&decoded,3));
         let data= *vector::borrow(&decoded,4);
         let protocols= decoder::decode_strings(vector::borrow(&decoded,5));
         let req=create(from,to,sn,message_type,data,protocols);
         req

    }

    

    public fun msg_type(req:&CSMessageRequest):u8 {
         req.message_type
    }

    public fun from(req:&CSMessageRequest):NetworkAddress {
        req.from
    }

    public fun to(req:&CSMessageRequest):String {
        req.to
    }

    public fun sn(req:&CSMessageRequest):u128 {
        req.sn
    }

    public fun data(req:&CSMessageRequest):vector<u8> {
        req.data
    }

    public fun protocols(req:&CSMessageRequest):vector<String> {
        req.protocols
    }

    public fun from_nid(req:&CSMessageRequest):String {
        network_address::net_id(&req.from)
    }

}

module xcall::message_request_tests {
    use xcall::network_address::{Self};
    use xcall::message_request::{Self};
    use std::string;
    use std::vector;
    use std::debug;
    use xcall::call_message::{Self};
    use xcall::call_message_rollback::{Self};
     use sui_rlp::encoder;
    use sui_rlp::decoder;
    /*
    CSMessageRequest
     from: 0x1.ETH/0xa
     to: cx0000000000000000000000000000000000000102
     sn: 21
     rollback: false
     data: 74657374
     protocol: []
     RLP: F83F8B3078312E4554482F307861AA63783030303030303030303030303030303030303030303030303030303030303030303030303031303215008474657374C0

     
     */

    #[test]
     fun test_message_request_encode_case_1(){
        let from=network_address::create(string::utf8(b"0x1.ETH"),string::utf8(b"0xa"));
        let network_bytes=network_address::encode(&from);
       


        let msg_request=message_request::create(from,
        string::utf8(b"cx0000000000000000000000000000000000000102"),
        21,
        call_message::msg_type(),
         x"74657374",
         vector::empty());
         let encoded_bytes=message_request::encode(&msg_request);
        
         assert!(encoded_bytes==x"F83F8B3078312E4554482F307861AA63783030303030303030303030303030303030303030303030303030303030303030303030303031303215008474657374C0",0x01);
        let decoded= message_request::decode(&encoded_bytes);
       
        assert!(decoded==msg_request,0x01);
        

     }
/*
CSMessageRequest
     from: 0x1.ETH/0xa
     to: cx0000000000000000000000000000000000000102
     sn: 21
     rollback: false
     data: 74657374
     protocol: [abc, cde, efg]
     RLP: F84B8B3078312E4554482F307861AA63783030303030303030303030303030303030303030303030303030303030303030303030303031303215008474657374CC836162638363646583656667

*/

     #[test]
     fun test_message_request_encode_case_2(){
        let from=network_address::create(string::utf8(b"0x1.ETH"),string::utf8(b"0xa"));
        let network_bytes=network_address::encode(&from);
        
        let mut protocols=vector::empty();
        protocols.push_back(string::utf8(b"abc"));
        protocols.push_back(string::utf8(b"cde"));
       
        protocols.push_back(string::utf8(b"efg"));
       


        let msg_request=message_request::create(from,
        string::utf8(b"cx0000000000000000000000000000000000000102"),
        21,
        call_message::msg_type(),
         x"74657374",
         protocols);
         let encoded_bytes=message_request::encode(&msg_request);
         assert!(encoded_bytes==x"F84B8B3078312E4554482F307861AA63783030303030303030303030303030303030303030303030303030303030303030303030303031303215008474657374CC836162638363646583656667",0x01);
         let decoded= message_request::decode(&encoded_bytes);
         
         assert!(decoded==msg_request,0x01);
        

     }

     /*
     CSMessageRequest
     from: 0x1.ETH/0xa
     to: cx0000000000000000000000000000000000000102
     sn: 21
     rollback: true
     data: 74657374
     protocol: [abc, cde, efg]
     RLP: F84B8B3078312E4554482F307861AA63783030303030303030303030303030303030303030303030303030303030303030303030303031303215018474657374CC836162638363646583656667


     
     */

     #[test]
     fun test_message_request_encode_case_3(){
        let from=network_address::create(string::utf8(b"0x1.ETH"),string::utf8(b"0xa"));
        let network_bytes=network_address::encode(&from);
        
        let mut protocols=vector::empty();
        protocols.push_back(string::utf8(b"abc"));
        protocols.push_back(string::utf8(b"cde"));
        protocols.push_back(string::utf8(b"efg"));

        let msg_request=message_request::create(from,
        string::utf8(b"cx0000000000000000000000000000000000000102"),
        21,
        call_message_rollback::msg_type(),
         x"74657374",
         protocols);

         let encoded_bytes=message_request::encode(&msg_request);
         assert!(encoded_bytes==x"F84B8B3078312E4554482F307861AA63783030303030303030303030303030303030303030303030303030303030303030303030303031303215018474657374CC836162638363646583656667",0x01);
         let decoded= message_request::decode(&encoded_bytes);
         assert!(decoded==msg_request,0x01);
        

     }



     
}