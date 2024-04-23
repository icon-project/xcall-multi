#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::message_response {
     use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
     use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;

   const RESPONSE_SUCCESS:u8=1;
   const RESPONSE_FAILURE:u8=0;
    
   

   

   

    

    public struct CSMessageResponse has store,drop{
        sn:u128,
        code:u8,
    }

    public fun new(sn:u128,code:u8):CSMessageResponse{
        CSMessageResponse {
            sn,
            code,
        }

    }

    public fun encode(val:&CSMessageResponse):vector<u8>{
         let mut list=vector::empty<vector<u8>>();
           
          vector::push_back(&mut list,encoder::encode_u128(val.sn));
          vector::push_back(&mut list,encoder::encode_u8(val.code));

          let encoded=encoder::encode_list(&list,false);
          encoded

    }


    public fun decode(bytes:&vector<u8>):CSMessageResponse {

         let decoded=decoder::decode_list(bytes);
         
         let sn= decoder::decode_u128(vector::borrow(&decoded,0));
         let code= decoder::decode_u8(vector::borrow(&decoded,1));
        
         let req=CSMessageResponse {
            sn,
            code
         };
         req

    }

    public fun success():u8 {
        RESPONSE_SUCCESS
    }

    public fun failure():u8 {
        RESPONSE_FAILURE
    }


   

}

module xcall::message_response_tests {
    use xcall::message_response::{Self};

     /*
    CSMessageResponse
     sn: 1
     code: CSMessageResponse.SUCCESS
     errorMessage: errorMessage
     RLP: C20101

     CSMessageResponse
     sn: 2
     code: CSMessageResponse.FAILURE
     errorMessage: errorMessage
     RLP: C20200
     */

      #[test]
    fun test_message_response_encoding_1(){
        let msg= message_response::new(1,message_response::success());
        let encoded= message_response::encode(&msg);
        assert!(encoded==x"C20101",0x01);
        let decoded=message_response::decode(&encoded);
        assert!(decoded==msg,0x01);


    }

     #[test]
    fun test_message_response_encoding_2(){
        let msg= message_response::new(2,message_response::failure());
        let encoded= message_response::encode(&msg);
        assert!(encoded==x"C20200",0x01);
        let decoded=message_response::decode(&encoded);
        assert!(decoded==msg,0x01);


    }






}