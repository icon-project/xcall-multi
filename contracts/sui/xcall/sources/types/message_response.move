#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::message_response {
     use std::string::{Self, String};
    use sui::object::{Self, ID, UID};
    use xcall::network_address::{Self,NetworkAddress};

   
    const CS_REQUEST:u8 = 1;

    const CS_RESULT:u8 = 2;
   

   

   

    

    struct CSMessageResponse has store{
        sn:u128,
        code:u8,
    }


   

}