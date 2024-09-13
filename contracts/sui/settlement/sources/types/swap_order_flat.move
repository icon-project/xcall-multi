module settlement::swap_order_flat {
    use std::string::{String,Self};
    use sui::event::{Self};
     use sui_rlp::encoder::{Self};
     use sui_rlp::decoder::{Self};
     public struct SwapOrderFlat has copy,drop,store {
    id:u128,                     
    src_nid:String,               
    dst_nid: String,             
    creator:vector<u8>,                
    destination_address:vector<u8>,    
    token:String,                  
    amount:u64,                 
    to_token:String,                 
    min_receive:u128,             
    data:vector<u8>,
}

public fun new( id:u128,                     
    src_nid:String,               
    dst_nid: String,             
    creator:vector<u8>,                
    destination_address:vector<u8>,    
    token:String,                  
    amount:u64,                 
    to_token:String,                 
    min_receive:u128,             
    data:vector<u8>):SwapOrderFlat{
        SwapOrderFlat {
            id,
            src_nid,
            dst_nid,
            creator,
            destination_address,
            token,
            amount,
            to_token,
            min_receive,
            data,
        }
    }




    public fun get_id(self:&SwapOrderFlat):u128 {
    self.id
}
public fun get_src_nid(self:&SwapOrderFlat):String {
    self.src_nid
}
public fun get_dst_nid(self:&SwapOrderFlat):String {
    self.dst_nid
}
public fun get_creator(self:&SwapOrderFlat):vector<u8> {
    self.creator
}
public fun get_destination_address(self:&SwapOrderFlat):vector<u8> {
    self.destination_address
}
public fun get_to_token(self:&SwapOrderFlat):String {
    self.to_token
}
public fun get_min_receive(self:&SwapOrderFlat):u128 {
    self.min_receive
}
public fun get_data(self:&SwapOrderFlat):&vector<u8> {
    &self.data
}

public fun get_token(self:&SwapOrderFlat):String {
    return self.token
}

public fun get_amount(self:&SwapOrderFlat):u64 {self.amount}
public fun emit(self:SwapOrderFlat){
    event::emit(self)
}

public(package) fun deduct_min_receive(self:&mut SwapOrderFlat,amount:u64){
    self.min_receive=((self.min_receive as u64)-amount) as u128;

}

public(package) fun deduct_amount(self:&mut SwapOrderFlat,amount:u64){
    self.amount=self.amount-amount;

}

public fun encode(self:&SwapOrderFlat):vector<u8>{
     let mut list=vector::empty<vector<u8>>();
           vector::push_back(&mut list,encoder::encode_u128(self.get_id()));
          vector::push_back(&mut list,encoder::encode_string(&self.get_src_nid()));
          vector::push_back(&mut list,encoder::encode_string(&self.get_dst_nid()));
          vector::push_back(&mut list,encoder::encode(&self.get_creator()));
           vector::push_back(&mut list,encoder::encode(&self.get_destination_address()));
           vector::push_back(&mut list,encoder::encode_string(&self.get_token()));
           vector::push_back(&mut list,encoder::encode_u64(self.get_amount()));
           vector::push_back(&mut list,encoder::encode_string(&self.get_to_token()));
           vector::push_back(&mut list,encoder::encode_u128(self.get_min_receive()));
            vector::push_back(&mut list,encoder::encode(self.get_data()));

          let encoded=encoder::encode_list(&list,false);
          encoded

}

public fun decode(bytes:&vector<u8>):SwapOrderFlat{
       let decoded=decoder::decode_list(bytes);
        let id= decoder::decode_u128(vector::borrow(&decoded,0));
        let src_nid= decoder::decode_string(decoded.borrow(1));
         let dst_nid= decoder::decode_string(decoded.borrow(2));
         let creator= *vector::borrow(&decoded,3);
         let destination_address=*vector::borrow(&decoded,3);
         let token= decoder::decode_string(decoded.borrow(4));
         let amount= decoder::decode_u64(decoded.borrow(5));
         let to_token=decoder::decode_string(decoded.borrow(6));
         let min_receive= decoder::decode_u128(decoded.borrow(7));
         let data= *decoded.borrow(8);
         SwapOrderFlat {
            id,
            src_nid,
            dst_nid,
            creator,
            destination_address,
            token,
            amount,
            to_token,
            min_receive,
            data,
         }

     


}
}

