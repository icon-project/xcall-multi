module settlement::order_fill {
    use sui_rlp::encoder::{Self};
    use sui_rlp::decoder::{Self};
    /// @title OrderFill type
    /// @notice Represents an order fill with the corresponding order ID, order hash, solver address, and fill amount.
    public struct OrderFill has copy,drop,store{
        id:u128,                      // ID of the order being filled
        order_bytes:vector<u8>,                 // Hash of the order
        solver:vector<u8>,                    // Address of the solver that fills the order
        amount:u128,                  // Amount filled by the solver
    }

    public fun new( id:u128,                      // ID of the order being filled
        order_bytes:vector<u8>,                 // Hash of the order
        solver:vector<u8>,                    // Address of the solver that fills the order
        amount:u128, ):OrderFill{
            OrderFill {
                id,
                order_bytes,
                solver,
                amount,
            }
        }

    public fun get_id(self:&OrderFill):u128{
        self.id
    }
    public fun get_order_bytes(self:&OrderFill):vector<u8>{
        self.order_bytes
    }
    public fun get_solver(self:&OrderFill):vector<u8>{
        self.solver
    }
    public fun get_amount(self:&OrderFill):u128{
        self.amount
    }

    public fun encode(self:&OrderFill):vector<u8>{
        let mut list=vector::empty<vector<u8>>();
        vector::push_back(&mut list,encoder::encode_u128(self.id));
        vector::push_back(&mut list,encoder::encode(&self.order_bytes));
        vector::push_back(&mut list,encoder::encode(&self.solver));
        vector::push_back(&mut list,encoder::encode_u128(self.amount));

        let encoded=encoder::encode_list(&list,false);
        encoded
    }

    public fun decode(bytes:&vector<u8>):OrderFill {
        let decoded=decoder::decode_list(bytes);
        let id= decoder::decode_u128(vector::borrow(&decoded,0));
        let order_bytes=  *vector::borrow(&decoded,1);
        let solver=  *vector::borrow(&decoded,2);
        let amount= decoder::decode_u128(vector::borrow(&decoded,3));

        OrderFill {
        id,
        order_bytes,
        solver,
        amount,
        }

    }



}