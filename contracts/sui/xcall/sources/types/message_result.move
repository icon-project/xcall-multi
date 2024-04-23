#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::message_result {
     use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};
    use std::vector;

    const CS_REQUEST:u8 = 1;

    const CS_RESULT:u8 = 2;

    struct CSMessageResponse has store, drop{
        sn:u128,
        code:u8,
        message: vector<u8>,
    }

    public fun create(sn:u128,code:u8,message:vector<u8>):CSMessageResponse {
        CSMessageResponse {
            sn:sn,
            code:code,
            message:message,
        }
    }

    public fun decode(bytes:vector<u8>):CSMessageResponse {
        CSMessageResponse {
            sn:0,
            code:0,
            message:vector::empty<u8>(),
        }
    }


    public fun sequence_no(self:&CSMessageResponse):u128 {
        self.sn
    }   


    public fun response_code(self:&CSMessageResponse):u8 {
        self.code
    }

    public fun message(self:&CSMessageResponse):vector<u8> {
        self.message
    }





   

}