module intents_v1::cluster_connection {
    use std::string::{String, Self};
    use sui::table::{Table, Self};
    use intents_v1::order_message::{OrderMessage, Self};
    use sui::event::{ Self };

    public struct Receipt has drop, copy, store {
        src_nid: String,
        conn_sn: u128,
    }

    public struct ConnectionState has store {
        conn_sn: u128,
        relayer: address, // Address of the relayer
        receipts: Table<Receipt, bool>, // Mapping of receipts for tracking
    }

    public struct Message has copy, drop {
        to: String,
        conn_sn: u128,
        msg: vector<u8>,
    }

    public(package) fun new(relayer: address, ctx: &mut TxContext): ConnectionState {
        ConnectionState {
            conn_sn: 0,
            relayer: relayer,
            receipts: table::new(ctx),
        }
    }

    public fun get_relayer(self: &ConnectionState): address {
        self.relayer
    }

    public(package) fun receive_message(
        self: &mut ConnectionState,
        srcNid: String,
        conn_sn: u128,
        msg: vector<u8>,
        ctx: &TxContext
    ): OrderMessage {
        assert!(self.relayer == ctx.sender());
        let key = Receipt {src_nid: srcNid, conn_sn};
        assert!(!self.receipts.contains(key));
        self.receipts.add(key, true);
        order_message::decode(&msg)
    }

    public(package) fun send_message(
        self: &mut ConnectionState,
        toNid: String,
        msg: vector<u8>
    ) {
        let conn_sn = get_next_conn_sn(self);
        event::emit(Message {to: toNid, conn_sn, msg,})

    }

    public(package) fun set_relayer(
        self: &mut ConnectionState,
        relayer: address
    ) {
        self.relayer = relayer;
    }

    fun get_next_conn_sn(self: &mut ConnectionState): u128 {
        let sn = self.conn_sn + 1;
        self.conn_sn = sn;
        sn
    }

     public fun get_receipt(self: &ConnectionState, net_id: String, sn: u128): bool {
        let receipt_key = Receipt { src_nid: net_id, conn_sn: sn };
        self.receipts.contains<Receipt,bool>(receipt_key)
    }
}
