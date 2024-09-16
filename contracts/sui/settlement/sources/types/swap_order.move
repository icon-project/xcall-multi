module settlement::swap_order {
    use std::string::{String,Self};
    use sui::coin::{Coin,Self};
    use std::type_name::{Self};
    use sui::event::{Self};
    use sui_rlp::encoder::{Self};
     use sui_rlp::decoder::{Self};
     use settlement::swap_order_flat::{SwapOrderFlat,Self};

    public struct SwapOrder<T:store> has store{
    id:u128,
    emitter:vector<u8>,                      // unique ID
    src_nid:String,                  // Source Network ID
    dst_nid: String,              // Destination Network ID
    creator:vector<u8>,                // The user who created the order
    destination_address:vector<u8>,     // Destination address on the destination network
    // token:address,                  // Token to be swapped
    // amount:u128,                 // Amount of the token to be swapped
    to_token:vector<u8>,                 // Token to receive on the destination network
    min_receive:u128,             // Minimum amount of the toToken to receive
    data:vector<u8>,
    token:Coin<T>                  // Additional data (if any) for future use
}

public fun new<T:store>( id: u128,emitter:vector<u8>,
            src_nid: String,
            dst_nid: String,
            creator: vector<u8>,
            destination_address: vector<u8>,
            token:Coin<T>,
           
            to_token: vector<u8>,
            min_receive: u128,
            data: vector<u8>):SwapOrder<T>
            {
                SwapOrder {
                    id,
                    emitter,
                    src_nid,
                    dst_nid,
                    creator,
                    destination_address,
                    to_token,
                    min_receive,
                    data,
                    token,
                }

            }


public fun get_id<T:store>(self:&SwapOrder<T>):u128 {
    self.id
}
public fun get_src_nid<T:store>(self:&SwapOrder<T>):String {
    self.src_nid
}
public fun get_dst_nid<T:store>(self:&SwapOrder<T>):String {
    self.dst_nid
}
public fun get_creator<T:store>(self:&SwapOrder<T>):vector<u8> {
    self.creator
}
public fun get_destination_address<T:store>(self:&SwapOrder<T>):vector<u8> {
    self.destination_address
}
public fun get_to_token<T:store>(self:&SwapOrder<T>):vector<u8> {
    self.to_token
}
public fun get_min_receive<T:store>(self:&SwapOrder<T>):u128 {
    self.min_receive
}
public fun get_data<T:store>(self:&SwapOrder<T>):&vector<u8> {
    &self.data
}
public fun get_token<T:store>(self:&SwapOrder<T>):&Coin<T> {
    &self.token
}

public fun encode<T:store>(self:&SwapOrder<T>):vector<u8>{
    let event=self.to_event();
    event.encode()
}




public fun to_event<T:store>(self:&SwapOrder<T>):SwapOrderFlat {
let event= swap_order_flat::new(
            self.id,
            self.emitter,
            self.src_nid,
            self.dst_nid,
            self.creator,
            self.destination_address,
            *string::from_ascii(type_name::get<T>().into_string()).as_bytes(),
            self.token.value() as u128,
            self.to_token,
            self.min_receive,
            self.data
);
    event
}

    public(package) fun fill_amount<T:store>(self:&mut SwapOrder<T>,amount:u64,ctx:&mut TxContext):Coin<T>{
        let split=coin::split(&mut self.token, amount, ctx);
        split


    }

    public(package) fun destroy<T:store>(self:SwapOrder<T>){
          let SwapOrder {
                    id,
                    emitter,
                    src_nid,
                    dst_nid,
                    creator,
                    destination_address,
                    to_token,
                    min_receive,
                    data,
                    token,
                } =self;
        coin::destroy_zero(token);
    }



   

}