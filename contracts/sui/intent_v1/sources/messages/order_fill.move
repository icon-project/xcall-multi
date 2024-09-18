module intents_v1::order_fill {
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    use std::string::{String,Self};
    /// @title OrderFill type
    /// @notice Represents an order fill with the corresponding order ID, order hash, solver address, and fill amount.
    public struct OrderFill has copy,drop,store{
        id:u128,                      // ID of the order being filled
        order_bytes:vector<u8>,                 // Hash of the order
        solver:String,                    // Address of the solver that fills the order
        amount:u128,                 // Amount filled by the solver
        close_order:bool,
    }

    public fun new( id:u128,                      // ID of the order being filled
        order_bytes:vector<u8>,                 // Hash of the order
        solver:String,                    // Address of the solver that fills the order
        amount:u128, close_order:bool):OrderFill{
            OrderFill {
                id,
                order_bytes,
                solver,
                amount,
                close_order
            }
        }

    public fun get_id(self:&OrderFill):u128{
        self.id
    }
    public fun get_order_bytes(self:&OrderFill):vector<u8>{
        self.order_bytes
    }
    public fun get_solver(self:&OrderFill):String{
        self.solver
    }
    public fun get_amount(self:&OrderFill):u128{
        self.amount
    }

    public fun get_close_order(self:&OrderFill):bool {
        self.close_order
    }

    public fun encode(self:&OrderFill):vector<u8>{
        let mut list=vector::empty<vector<u8>>();
        vector::push_back(&mut list,encoder::encode_u128(self.id));
        vector::push_back(&mut list,encoder::encode(&self.order_bytes));
        vector::push_back(&mut list,encoder::encode_string(&self.solver));
        vector::push_back(&mut list,encoder::encode_u128(self.amount));
         vector::push_back(&mut list,encoder::encode_bool(self.close_order));


        let encoded=encoder::encode_list(&list,false);
        encoded
    }

    public fun decode(bytes:&vector<u8>):OrderFill {
        let decoded=decoder::decode_list(bytes);
        let id= decoder::decode_u128(vector::borrow(&decoded,0));
        let order_bytes=  *vector::borrow(&decoded,1);
        let solver=  decoder::decode_string(vector::borrow(&decoded,2));
        let amount= decoder::decode_u128(vector::borrow(&decoded,3));
         let close_order= decoder::decode_bool(vector::borrow(&decoded,4));

        OrderFill {
        id,
        order_bytes,
        solver,
        amount,
        close_order,
        }

    }

 #[test]
 fun test_order_fill_encoding(){
    let swap_order= OrderFill {
      id: 1,
    order_bytes: x"6c449988e2f33302803c93f8287dc1d8cb33848a",
    solver: string::utf8(b"0xcb0a6bbccfccde6be9f10ae781b9d9b00d6e63"),
    amount: 500,
    close_order: true
    };

    let encoded= swap_order.encode();
    std::debug::print(&encoded);
    assert!(encoded==x"f84301946c449988e2f33302803c93f8287dc1d8cb33848aa8307863623061366262636366636364653662653966313061653738316239643962303064366536338201f401")

 }



 #[test]
 fun test_order_fill_encoding2(){
    let swap_order= OrderFill {
      id: 2,
    order_bytes: x"cb0a6bbccfccde6be9f10ae781b9d9b00d6e63",
    solver: string::utf8(b"0x6c449988e2f33302803c93f8287dc1d8cb33848a"),
    amount: 750 * 1000000000000000000,
    close_order: false
    };

    let encoded= swap_order.encode();
    std::debug::print(&encoded);
    assert!(encoded==x"f84b0293cb0a6bbccfccde6be9f10ae781b9d9b00d6e63aa3078366334343939383865326633333330323830336339336638323837646331643863623333383438618928a857425466f8000000")

 }

  



}