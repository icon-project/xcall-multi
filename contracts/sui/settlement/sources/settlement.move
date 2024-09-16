
/// Module: settlement
module settlement::main {
    use std::string::{String, Self};
    use sui::linked_table::{LinkedTable, Self};
    use sui::table::{Table, Self};
    use sui::transfer::{ Self };
    use sui::coin::{Coin, Self};
    use sui::bag::{Bag, Self};
    use sui::event::{ Self };
    use std::type_name::{ Self };
    use settlement::order_fill::{OrderFill, Self};
    use settlement::order_cancel::{Cancel, Self};
    use settlement::order_message::{OrderMessage, Self};
    use settlement::swap_order::{Self, SwapOrder};
    use settlement::swap_order_flat::{Self, SwapOrderFlat};
    use sui::hash::keccak256;
    use sui::address::{Self as suiaddress};
    use settlement::cluster_connection::{Self, ConnectionState};

    const FILL: u8 = 1; // Constant for Fill message type
    const CANCEL: u8 = 2; // Constant for Cancel message type
    const CURRENT_VERSION: u64 = 1;

    const EAlreadyFinished: u64 = 1;

    public struct Receipt has drop, copy, store {
        src_nid: String,
        conn_sn: u128,
    }

    public struct AdminCap has key, store {
        id: UID
    }

    public struct Storage has key, store {
        id: UID,
        version: u64,
        deposit_id: u128, // Deposit ID counter
        nid: String, // Network Identifier
        connection: ConnectionState,
        orders: Table<u128, SwapOrderFlat>, // Mapping of deposit ID to SwapOrder
        pending_fills: Table<vector<u8>, u128>, // Mapping of order hash to pending payment
        finished_orders: Table<vector<u8>, bool>,
        fee: u8,
        fee_handler: address,
        funds:Bag,
    }

    fun init(ctx: &mut TxContext) {
        let admin = AdminCap {id: object::new(ctx)};
        let storage = Storage {
            id: object::new(ctx),
            version: CURRENT_VERSION,
            deposit_id: 0,
            nid: string::utf8(b"sui"),
            connection: cluster_connection::new(ctx.sender(), ctx),
            orders: table::new(ctx),
            pending_fills: table::new(ctx),
            finished_orders: table::new(ctx),
            fee: 1,
            fee_handler: ctx.sender(),
            funds:bag::new(ctx),

        };
        transfer::public_transfer(admin, ctx.sender());
        transfer::public_share_object(storage);

    }

    fun get_deposit_id(storage: &mut Storage): u128 {
        let deposit_id = storage.deposit_id + 1;
        storage.deposit_id = deposit_id;
        deposit_id
    }

    fun get_connection_state_mut(self: &mut Storage): &mut ConnectionState {
        &mut self.connection
    }

    entry fun swap<T: store>(
        self: &mut Storage,
        toNid: String,
        token: Coin<T>,
        toToken: String,
        toAddress: vector<u8>,
        minReceive: u128,
        data: vector<u8>,
        ctx: &TxContext

    ) {
        // Escrows amount from user
        let deposit_id = get_deposit_id(self);
        let order = swap_order_flat::new(
            deposit_id,
            self.id.to_bytes(),
            self.nid,
            toNid,
            ctx.sender().to_bytes(),
            toAddress,
     *string::from_ascii(type_name::get<T>().into_string()).as_bytes(),
            token.value() as u128,
            *toToken.as_bytes(),
            minReceive,
            data
        );
        self.funds.add<u128,Coin<T>>(deposit_id, token);

        swap_order_flat::emit(order);
        self.orders.add(deposit_id, order);

    }

    entry fun recv_message<T: store>(
        self: &mut Storage,
        srcNetwork: String,
        conn_sn: u128,
        msg: vector<u8>,
        ctx: &mut TxContext,
    ) {
        let orderMessage = cluster_connection::receive_message(
            self.get_connection_state_mut(),
            srcNetwork,
            conn_sn,
            msg,
            ctx
        );
        if (orderMessage.get_type() == FILL) {
            let fill = order_fill::decode(&orderMessage.get_message());
            resolve_fill<T>(self, srcNetwork, &fill, ctx);
        }
        else if (orderMessage.get_type() == CANCEL) {
            let cancel = order_cancel::decode(&orderMessage.get_message());
            resolve_cancel(self, cancel.get_order_bytes(), ctx);
        }
    }

    fun resolve_fill<T: store>(
        self: &mut Storage,
        srcNid: String,
        fill: &OrderFill,
        ctx: &mut TxContext
    ) {
        let order = self.orders.borrow<u128, SwapOrderFlat>(fill.get_id());

        assert!(keccak256(&order.encode()) == keccak256(&fill.get_order_bytes()));
        assert!(order.get_dst_nid() == srcNid);

        let take= {
            let fund= self.funds.borrow_mut<u128,Coin<T>>(fill.get_id());
            assert!(fund.value() >= (fill.get_amount() as u64));
            let take = fund.split((fill.get_amount() as u64), ctx);
            take
        };

        if (fill.get_close_order() == true) {
            self.orders.remove<u128, SwapOrderFlat>(fill.get_id());
            coin::destroy_zero(self.funds.remove<u128,Coin<T>>(fill.get_id()));
        };

        let solver = suiaddress::from_bytes(fill.get_solver());
        transfer::public_transfer(take, solver);
    }

    fun resolve_cancel(
        self: &mut Storage,
        order_bytes: vector<u8>,
        ctx: &TxContext
    ) {
        let order_hash = keccak256(&order_bytes);
        let order = swap_order_flat::decode(&order_bytes);

        if (self.finished_orders.contains(order_hash)) {
            abort EAlreadyFinished
        };

        let pending_fill = self.pending_fills.remove<vector<u8>, u128>(order_hash);
        self.finished_orders.add<vector<u8>, bool>(order_hash, true);

        let orderFill = order_fill::new(
            order.get_id(),
            order_bytes,
            order.get_creator(),
            pending_fill,
            true
        );

        let msg = order_message::new(FILL, orderFill.encode());
        cluster_connection::send_message(
            self.get_connection_state_mut(),
            order.get_src_nid(),
            msg.encode()
        )
    }

    /// @notice Fills an order for a cross-chain swap.
    /// @param id The order ID.
    /// @param order The SwapOrder object.
    /// @param amount The amount to fill.
    /// @param solverAddress The address of the solver filling the order.
    entry fun fill<T: store>(
        self: &mut Storage,
        id: u128,
        order_bytes: vector<u8>,
        mut fill_token: Coin<T>,
        solveraddress: address,
        ctx: &mut TxContext
    ) {
        let order_hash = keccak256(&order_bytes);
        let order = swap_order_flat::decode(&order_bytes);

        assert!(!self.finished_orders.contains(order_hash));

        // make sure user is filling token wanted by order
        assert!(*string::from_ascii(type_name::get<T>().into_string()).as_bytes() == order.get_to_token());
        assert!((fill_token.value() as u128) <= order.get_min_receive());

        // insert order if its first occurrence
        if (!self.pending_fills.contains(order_hash)) {
            self.pending_fills.add<vector<u8>,u128>(order_hash, order.get_amount());
        };

        let payout = (order.get_amount() * (fill_token.value() as u128)) / order.get_min_receive();
        let mut pending = self.pending_fills.remove<vector<u8>, u128>(order_hash);
        pending = pending - payout;

        if (pending == 0) {
            self.finished_orders.add(order_hash, true);
        }else{
            self.pending_fills.add(order_hash, pending);
        };



        let fee = (fill_token.value() * (self.fee as u64)) / 10000;
        let fee_token = fill_token.split(fee, ctx);

        let fill = order_fill::new(
            id,
            order_bytes,
            solveraddress.to_bytes(),
            payout,
            self.finished_orders.contains(order_hash)
        );
        let msg = order_message::new(FILL, fill.encode());

        transfer::public_transfer(
            fill_token,
            suiaddress::from_bytes(order.get_destination_address())
        );
        transfer::public_transfer(fee_token, self.fee_handler);


        if (order.get_src_nid() == order.get_dst_nid()) {
            self.resolve_fill<T>(order.get_src_nid(), &fill, ctx);
        } else {
            cluster_connection::send_message(
                self.get_connection_state_mut(),
                order.get_src_nid(),
                msg.encode()
            );
        };

    }

    entry fun cancel<T: store>(
        self: &mut Storage,
        id: u128,
        ctx: &TxContext
    ) {
        let (msg, src_nid, dst_nid) = {
            let order = self.orders.borrow<u128, SwapOrderFlat>(id);
            assert!(
                order.get_creator() == ctx.sender().to_bytes()
            );
            let msg = order_cancel::new(order.encode());
            let order_msg = order_message::new(CANCEL, msg.encode());
            (
                order_msg,
                order.get_src_nid(),
                order.get_dst_nid()
            )
        };
        if (src_nid == dst_nid) {
            self.resolve_cancel(msg.encode(), ctx);
        } else {
            cluster_connection::send_message(
                self.get_connection_state_mut(),
                dst_nid,
                msg.encode()
            );
        };

    }

    // admin functions //

    entry fun set_relayer(
        self: &mut Storage,
        _cap: &AdminCap,
        relayer: address
    ) {
        self.get_connection_state_mut().set_relayer(relayer);
    }

}
