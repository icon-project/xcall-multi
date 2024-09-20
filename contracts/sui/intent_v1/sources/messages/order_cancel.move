module intents_v1::order_cancel {
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    /// @title Cancel type
    /// @notice Represents a cancellation of an order with the corresponding order hash.
    public struct Cancel has copy,drop{
        order_bytes:vector<u8>,                 // Hash of the order to be canceled
    }

    public fun new(order_bytes:vector<u8>):Cancel {
        Cancel { order_bytes }
    }

    public fun get_order_bytes(self:&Cancel): vector<u8>{
        self.order_bytes
    }

    public fun encode(self:Cancel):vector<u8>{
        let mut list=vector::empty<vector<u8>>();
        vector::push_back(&mut list,encoder::encode(&self.order_bytes));

        let encoded=encoder::encode_list(&list,false);
        encoded

    }

    public fun decode(bytes:&vector<u8>):Cancel{
        let decoded=decoder::decode_list(bytes);
        let data=  *vector::borrow(&decoded,0);

        Cancel {
        order_bytes:data,
        }
    }

#[test]
 fun test_order_cancel_encoding(){
    let swap_order= Cancel {
      order_bytes:x"6c449988e2f33302803c93f8287dc1d8cb33848a",
    };

    let encoded= swap_order.encode();
    std::debug::print(&encoded);
    assert!(encoded==x"d5946c449988e2f33302803c93f8287dc1d8cb33848a")

 }

}