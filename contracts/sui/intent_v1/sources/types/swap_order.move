module intents_v1::swap_order {
    use std::string::{String,Self};
    use sui::event::{Self};
     use sui_rlp::encoder::{Self};
     use sui::hex::{Self};
     use sui_rlp::decoder::{Self};
     public struct SwapOrder has copy,drop,store {
        id:u128,   
        emitter:String,                
        src_nid:String,               
        dst_nid: String,             
        creator:String,                
        destination_address:String,    
        token:String,                
        amount:u128,                 
        to_token:String,                 
        to_amount:u128,             
        data:vector<u8>,
}

public fun new( id:u128,    
    emitter:String,                 
    src_nid:String,               
    dst_nid: String,             
    creator:String,                
    destination_address:String,    
    token:String,                  
    amount:u128,                 
    to_token:String,                 
    to_amount:u128,             
    data:vector<u8>):SwapOrder{
        SwapOrder {
            id,
            emitter,
            src_nid,
            dst_nid,
            creator,
            destination_address,
            token,
            amount,
            to_token,
            to_amount,
            data,
        }
    }




    public fun get_id(self:&SwapOrder):u128 {
    self.id
}

public fun get_emitter(self:&SwapOrder):String{
    self.emitter
}
public fun get_src_nid(self:&SwapOrder):String {
    self.src_nid
}
public fun get_dst_nid(self:&SwapOrder):String {
    self.dst_nid
}
public fun get_creator(self:&SwapOrder):String {
    self.creator
}
public fun get_destination_address(self:&SwapOrder):String{
    self.destination_address
}
public fun get_to_token(self:&SwapOrder):String {
    self.to_token
}
public fun get_to_amount(self:&SwapOrder):u128 {
    self.to_amount
}
public fun get_data(self:&SwapOrder):&vector<u8> {
    &self.data
}

public fun get_token(self:&SwapOrder):String {
    return self.token
}

public fun get_amount(self:&SwapOrder):u128 {self.amount}

public(package) fun emit(self:SwapOrder){
    event::emit(self)
}

public(package) fun deduct_to_amount(self:&mut SwapOrder,amount:u128){
    self.to_amount=self.to_amount-amount

}

public(package) fun deduct_amount(self:&mut SwapOrder,amount:u128){
    self.amount=self.amount-amount;

}

public fun encode(self:&SwapOrder):vector<u8>{
     let mut list=vector::empty<vector<u8>>();
           vector::push_back(&mut list,encoder::encode_u128(self.get_id()));
            vector::push_back(&mut list,encoder::encode_string(&self.get_emitter()));
          vector::push_back(&mut list,encoder::encode_string(&self.get_src_nid()));
          vector::push_back(&mut list,encoder::encode_string(&self.get_dst_nid()));
          vector::push_back(&mut list,encoder::encode_string(&self.get_creator()));
            vector::push_back(&mut list,encoder::encode_string(&self.get_destination_address()));
           vector::push_back(&mut list,encoder::encode_string(&self.get_token()));
           vector::push_back(&mut list,encoder::encode_u128(self.get_amount()));
         vector::push_back(&mut list,encoder::encode_string(&self.get_to_token()));
           vector::push_back(&mut list,encoder::encode_u128(self.get_to_amount()));
            vector::push_back(&mut list,encoder::encode(self.get_data()));

          let encoded=encoder::encode_list(&list,false);
          encoded

}

public fun decode(bytes:&vector<u8>):SwapOrder{
       let decoded=decoder::decode_list(bytes);
        let id= decoder::decode_u128(vector::borrow(&decoded,0));
         let emitter= decoder::decode_string(decoded.borrow(1));
        let src_nid= decoder::decode_string(decoded.borrow(2));
         let dst_nid= decoder::decode_string(decoded.borrow(3));
         let creator= decoder::decode_string(decoded.borrow(4));
         let destination_address=decoder::decode_string(decoded.borrow(5));
         let token= decoder::decode_string(decoded.borrow(6));
         let amount= decoder::decode_u128(decoded.borrow(7));
         let to_token=decoder::decode_string(decoded.borrow(8));
         let to_amount= decoder::decode_u128(decoded.borrow(9));
         let data= *decoded.borrow(10);
         SwapOrder {
            id,
            emitter,
            src_nid,
            dst_nid,
            creator,
            destination_address,
            token,
            amount,
            to_token,
            to_amount,
            data,
         }

     


}

/*

0xf8880193cc7936ea419516635fc6feb8ad2d41b5d0c2b3894e6574776f726b2d31894e6574776f726b2d3293cc7936ea419516635fc6feb8ad2d41b5d0c2b393cc7936ea419516635fc6feb8ad2d41b5d0c2b393cc7936ea419516635fc6feb8ad2d41b5d0c2b38300afc893cc7936ea419516635fc6feb8ad2d41b5d0c2b3893635c9adc5dea0000080
Types.SwapOrder memory orderEncodingTest = Types.SwapOrder({
            id: 1,
            emitter: hex"CC7936eA419516635fC6fEb8AD2d41b5D0C2B3",
            srcNID: "Network-1",
            dstNID: "Network-2",
            creator: hex"CC7936eA419516635fC6fEb8AD2d41b5D0C2B3",
            destinationAddress: hex"CC7936eA419516635fC6fEb8AD2d41b5D0C2B3",
            token:hex"CC7936eA419516635fC6fEb8AD2d41b5D0C2B3",
            amount: 250*10**18,
            toToken: hex"CC7936eA419516635fC6fEb8AD2d41b5D0C2B3",
            minReceive: 1000'*10**18,
            data: hex""
        }); 

*/

#[test]
 fun test_swap_order_encoding(){
    let swap_order= SwapOrder {
        id:1,
        emitter:string::utf8(b"0xbe6452d4d6c61cee97d3"),
        src_nid:string::utf8(b"Ethereum"),
        dst_nid:string::utf8(b"Polygon"),
        creator:string::utf8(b"0x3e36eddd65e239222e7e67"),
        destination_address:string::utf8(b"0xd2c6218b875457a41b6fb7964e"),
        token:string::utf8(b"0x14355340e857912188b7f202d550222487"),
        amount:1000,
        to_token:string::utf8(b"0x91a4728b517484f0f610de7b"),
        to_amount:900,
        data:x"",
    };

    let encoded= swap_order.encode();
    assert!(encoded==x"f8a601963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378203e89a307839316134373238623531373438346630663631306465376282038480")

 }

 #[test]
 fun test_swap_order_encoding2(){
    let swap_order= SwapOrder {
        id:1,
        emitter:string::utf8(b"0xbe6452d4d6c61cee97d3"),
        src_nid:string::utf8(b"Ethereum"),
        dst_nid:string::utf8(b"Polygon"),
        creator:string::utf8(b"0x3e36eddd65e239222e7e67"),
        destination_address:string::utf8(b"0xd2c6218b875457a41b6fb7964e"),
        token:string::utf8(b"0x14355340e857912188b7f202d550222487"),
        amount:100000*10000000000000000000000,
        to_token:string::utf8(b"0x91a4728b517484f0f610de7b"),
        to_amount:900*10000000,
        data:x"6c449988e2f33302803c93f8287dc1d8cb33848a",
    };

    let encoded= swap_order.encode();
    assert!(encoded==x"f8c701963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378c033b2e3c9fd0803ce80000009a3078393161343732386235313734383466306636313064653762850218711a00946c449988e2f33302803c93f8287dc1d8cb33848a")

 }
}

