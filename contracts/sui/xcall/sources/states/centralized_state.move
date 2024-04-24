module xcall::centralized_state {
    use std::string::{Self, String};
    use sui::vec_map::{Self, VecMap};
     use xcall::xcall_utils::{Self as utils};
    /**
     message_fee: Map<'a, NetId, u128>,
    response_fee: Map<'a, NetId, u128>,
    admin: Item<'a, Addr>,
    conn_sn: Item<'a, u128>,
    receipts: Map<'a, (String, u128), bool>,
    xcall: Item<'a, Addr>,
    */
    

    public struct ReceiptKey has copy, drop, store {
        conn_sn: u128,
        nid: String,
    }

    public struct State has store { 
        message_fee: VecMap<String, u64>,
        response_fee: VecMap<String, u64>,
        receipts: VecMap<ReceiptKey, bool>,
        xcall: String,
        admin: String,
        conn_sn: u128
    }

    public fun create(): State {
        State {
            message_fee: vec_map::empty<String, u64>(),
            response_fee: vec_map::empty<String, u64>(),
            conn_sn: 0,
            receipts: vec_map::empty(),
            xcall: string::utf8(b""), 
            admin: string::utf8(b"")
        }
    }

    public(package) fun get_next_conn_sn(self:&mut State):u128 {
        let sn=self.conn_sn+1;
        self.conn_sn=sn;
        sn
    }

    public fun get_fee(self: &State, netId: &String, response: bool): u64 {
        let fee: u64 = if (response == true) {
            utils::get_or_default(&self.message_fee, netId, 0)
                + utils::get_or_default(&self.response_fee, netId, 0)
        } else {
            utils::get_or_default(&self.message_fee, netId, 0)
        };
        fee
    }

    public(package) fun set_fee(self: &mut State, net_id: String, message_fee: u64, response_fee: u64) {
        vec_map::insert(&mut self.message_fee, net_id, message_fee);
        vec_map::insert(&mut self.response_fee, net_id, response_fee);
    }

    public(package) fun check_duplicate_message(self: &mut State, net_id: String, sn: u128) {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        assert!(!vec_map::contains(&self.receipts, &receipt_key), 100);
        vec_map::insert(&mut self.receipts, receipt_key, true);
    }

    public(package) fun get_receipt(self: &State, net_id: String, sn: u128): bool {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        vec_map::contains(&self.receipts, &receipt_key)
    }
}