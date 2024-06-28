#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::message_result {
    use std::string::{Self, String};
    use xcall::network_address::{Self,NetworkAddress};
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};


    const RESPONSE_SUCCESS:u8=1;
    const RESPONSE_FAILURE:u8=0;

    public struct CSMessageResult has store, drop{
        sn:u128,
        code:u8,
        message: vector<u8>,
    }

    public fun create(sn:u128,code:u8,message:vector<u8>):CSMessageResult {
        CSMessageResult {
            sn:sn,
            code:code,
            message:message,
        }
    }


    public fun sequence_no(self:&CSMessageResult):u128 {
        self.sn
    }   


    public fun response_code(self:&CSMessageResult):u8 {
        self.code
    }

    public fun message(self:&CSMessageResult):vector<u8> {
        self.message
    }

    public fun encode(self:&CSMessageResult):vector<u8>{
        let mut list=vector::empty<vector<u8>>();

        vector::push_back(&mut list,encoder::encode_u128(self.sn));
        vector::push_back(&mut list,encoder::encode_u8(self.code));
        vector::push_back(&mut list,encoder::encode(&self.message));

        let encoded=encoder::encode_list(&list,false);
        encoded

    }


    public fun decode(bytes:&vector<u8>):CSMessageResult {

        let decoded=decoder::decode_list(bytes);

        let sn= decoder::decode_u128(vector::borrow(&decoded,0));
        let code= decoder::decode_u8(vector::borrow(&decoded,1));
        let message= *vector::borrow(&decoded,2);

        let req=CSMessageResult {
            sn,
            code,
            message,
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

module xcall::message_result_tests {
    use xcall::message_result::{Self};
    use std::debug::{Self};


    #[test]
    fun test_message_result_encoding_1(){
        let msg= message_result::create(1,message_result::success(),vector::empty());
        let encoded= message_result::encode(&msg);
        std::debug::print(&encoded);
        assert!(encoded==x"c58200010180",0x01);
        let decoded=message_result::decode(&encoded);
        assert!(decoded==msg,0x01);


    }

    #[test]
    fun test_message_result_encoding_2(){
        let msg= message_result::create(2,message_result::failure(),vector::empty());
        let encoded= message_result::encode(&msg);
        std::debug::print(&encoded);
        assert!(encoded==x"c58200020080",0x01);
        let decoded=message_result::decode(&encoded);
        assert!(decoded==msg,0x01);


    }






}