#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::cs_message {
    use std::string::{Self, String};
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::message_request::{Self,CSMessageRequest};
    use xcall::message_result::{Self,CSMessageResult};
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::debug;

    const CS_REQUEST:u8 = 1;

    const CS_RESULT:u8 = 2;

    public struct CSMessage has store,drop{
        msg_type:u8,
        payload:vector<u8>,
    }

    public fun new(msg_type:u8,payload:vector<u8>):CSMessage{
        CSMessage {
            msg_type,
            payload
        }
    }

    public fun from_message_request(req:CSMessageRequest):CSMessage {
        CSMessage {
            msg_type:CS_REQUEST,
            payload:message_request::encode(&req),
        }
    }

    public fun from_message_result(req:CSMessageResult):CSMessage {
        CSMessage {
            msg_type:CS_RESULT,
            payload:message_result::encode(&req),
        }
    }


    public fun msg_type( msg:&CSMessage):u8 {
        msg.msg_type
    }

    public fun payload( msg:&CSMessage):vector<u8> {
        msg.payload
    }

     public fun encode(req:&CSMessage):vector<u8>{
          let mut list=vector::empty<vector<u8>>();
           
          vector::push_back(&mut list,encoder::encode_u8(req.msg_type));
          vector::push_back(&mut list,encoder::encode(&req.payload));

          let encoded=encoder::encode_list(&list,false);
          encoded
    }

    public fun decode(bytes:&vector<u8>):CSMessage {
         let decoded=decoder::decode_list(bytes);
         
         let message_type= decoder::decode_u8(vector::borrow(&decoded,0));
         let payload= *vector::borrow(&decoded,1);
        
         let req=CSMessage {
            msg_type:message_type,
            payload
         };
         req

    }

    public fun request_code():u8 {
        CS_REQUEST
    }

    public fun result_code():u8 {
        CS_RESULT
    }
}

#[test_only]
module xcall::cs_message_tests {
    use xcall::cs_message::{Self};

      /*
        CSMessage
        type: CSMessage.REQUEST
        data: 7465737431
        RLP: C701857465737431

        CSMessage
        type: CSMessage.RESPONSE
        data: 7465737431
        RLP: C702857465737431
    */

    #[test]
    fun test_cs_message_encoding_1(){
        let msg= cs_message::new(1,x"7465737431");
        let encoded= cs_message::encode(&msg);
        assert!(encoded==x"C701857465737431",0x01);
        let decoded=cs_message::decode(&encoded);
        assert!(decoded==msg,0x01);


    }

     #[test]
    fun test_cs_message_encoding_2(){
        let msg= cs_message::new(2,x"7465737431");
        let encoded= cs_message::encode(&msg);
        assert!(encoded==x"C702857465737431",0x01);
        let decoded=cs_message::decode(&encoded);
        assert!(decoded==msg,0x01);


    }




}