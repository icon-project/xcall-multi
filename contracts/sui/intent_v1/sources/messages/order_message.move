module intents_v1::order_message {
     use sui_rlp::encoder::{Self};
     use sui_rlp::decoder::{Self};
    public struct OrderMessage has copy,drop{
    message_type:u8,            // Message type: FILL or CANCEL
    message:vector<u8>                 // Encoded message content
}

public fun new(message_type:u8,message:vector<u8>):OrderMessage{
    OrderMessage { message_type, message }
}

public fun get_type(self:&OrderMessage):u8{
    self.message_type
}

public fun get_message(self:&OrderMessage):vector<u8>{self.message}


public fun encode(self:&OrderMessage):vector<u8>{
    let mut list=vector::empty<vector<u8>>();
    list.push_back(encoder::encode_u8(self.message_type));
    vector::push_back(&mut list,encoder::encode(&self.message));

    let encoded=encoder::encode_list(&list,false);
    encoded

}
 public fun decode(bytes:&vector<u8>):OrderMessage {
         let decoded=decoder::decode_list(bytes);
         let message_type= decoder::decode_u8(vector::borrow(&decoded,0));
         let message=  *vector::borrow(&decoded,1);
        
         OrderMessage{
            message_type,
            message
            
         }

    }


    #[test]
 fun test_order_message_encoding(){
    let swap_order= OrderMessage {
    message_type: 1,
    message: x"6c449988e2f33302803c93f8287dc1d8cb33848a"
   };


    let encoded= swap_order.encode();
    std::debug::print(&encoded);
    assert!(encoded==x"d601946c449988e2f33302803c93f8287dc1d8cb33848a")

 }

 #[test]
 fun test_order_message_encoding2(){
    let swap_order= OrderMessage {
    message_type: 2,
    message: x"6c449988e2f33302803c93f8287dc1d8cb33848a"
   };


    let encoded= swap_order.encode();
    std::debug::print(&encoded);
    assert!(encoded==x"d602946c449988e2f33302803c93f8287dc1d8cb33848a")

 }

}