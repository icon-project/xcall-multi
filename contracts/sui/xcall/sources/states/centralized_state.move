module xcall::centralized_state {
    use std::string::{Self, String};
    use sui::vec_map::{Self, VecMap};
    use xcall::utils::{Self};

    friend xcall::centralized_connection;

    struct ReceiptKey has copy, drop, store {
        conn_sn: u128,
        nid: String,
    }

    struct State has store { 
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

    public(friend) fun get_next_conn_sn(self:&mut State):u128 {
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

    public(friend) fun set_fee(self: &mut State, net_id: String, message_fee: u64, response_fee: u64) {
        vec_map::insert(&mut self.message_fee, net_id, message_fee);
        vec_map::insert(&mut self.response_fee, net_id, response_fee);
    }

    public(friend) fun check_duplicate_message(self: &mut State, net_id: String, sn: u128) {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        assert!(!vec_map::contains(&self.receipts, &receipt_key), 100);
        vec_map::insert(&mut self.receipts, receipt_key, true);
    }

    public(friend) fun get_receipt(self: &State, net_id: String, sn: u128): bool {
        let receipt_key = ReceiptKey { nid: net_id, conn_sn: sn };
        vec_map::contains(&self.receipts, &receipt_key)
    }
}