
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
    use settlement::cluster_connection::{Self,ConnectionState};

    const FILL: u8 = 1; // Constant for Fill message type
    const CANCEL: u8 = 2; // Constant for Cancel message type
    const CURRENT_VERSION: u64 = 1;

    const EAlreadyFinished:u64=1;

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
        connection:ConnectionState,
        orders: Bag, // Mapping of deposit ID to SwapOrder
        pending_fills: Bag, // Mapping of order hash to pending SwapOrder fills
        finished_orders: Table<vector<u8>, bool>
    }

    fun init(ctx: &mut TxContext) {
        let admin = AdminCap {id: object::new(ctx)};
        let storage = Storage {
            id: object::new(ctx),
            version: CURRENT_VERSION,
            deposit_id: 0,
            nid: string::utf8(b"sui"),
            connection:cluster_connection::new(ctx.sender(),ctx),
            orders: bag::new(ctx),
            pending_fills: bag::new(ctx),
            finished_orders: table::new(ctx),
        };
        transfer::public_transfer(admin, ctx.sender());
        transfer::public_share_object(storage);

    }

    fun get_deposit_id(storage: &mut Storage): u128 {
        let deposit_id = storage.deposit_id + 1;
        storage.deposit_id = deposit_id;
        deposit_id
    }

    fun get_connection_state_mut(self:&mut Storage):&mut ConnectionState {
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

        let order = swap_order::new<T>(
            deposit_id,
            self.nid,
            toNid,
            ctx.sender().to_bytes(),
            toAddress,
            token,

            toToken,
            minReceive,
            data
        );

        swap_order_flat::emit(order.to_event());
        self.orders.add(deposit_id, order);

    }

    entry fun recv_message<T: store>(
        self: &mut Storage,
        srcNetwork: String,
        conn_sn: u128,
        msg: vector<u8>,
        ctx: &mut TxContext,
    ) {
        let orderMessage = cluster_connection::receive_message(self.get_connection_state_mut(), srcNetwork, conn_sn, msg, ctx);
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
        let order = self.orders.borrow_mut<u128, SwapOrder<T>>(fill.get_id());
        assert!(
            keccak256(&order.encode()) == keccak256(&fill.get_order_bytes())
        );
        assert!(order.get_dst_nid() == srcNid);
        assert!(
            order.get_token().value() >= (fill.get_amount() as u64)
        );
        let take = order.fill_amount((fill.get_amount() as u64), ctx);

        if (order.get_token().value() == 0) {
            let order=self.orders.remove<u128,SwapOrder<T>>(fill.get_id());
            swap_order::destroy(order);
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
        if (self.finished_orders.contains(order_hash)) {
            abort EAlreadyFinished
        };
        let pending_fill = *self.pending_fills.borrow<vector<u8>, SwapOrderFlat>(order_hash);
        self.pending_fills.remove<vector<u8>, SwapOrderFlat>(order_hash);
        self.finished_orders.add<vector<u8>, bool>(order_hash, true);

        let orderFill = order_fill::new(
            pending_fill.get_id(),
            order_bytes,
            pending_fill.get_creator(),
            (pending_fill.get_amount() as u128)
        );

        let msg = order_message::new(FILL, orderFill.encode());
        cluster_connection::send_message(self.get_connection_state_mut(), pending_fill.get_src_nid(), msg.encode())
    }

    /// @notice Fills an order for a cross-chain swap.
    /// @param id The order ID.
    /// @param order The SwapOrder object.
    /// @param amount The amount to fill.
    /// @param solverAddress The address of the solver filling the order.
    entry fun fill<T>(
        self: &mut Storage,
        id: u128,
        order_bytes: vector<u8>,
        fill_token: Coin<T>,
        solveraddress: address,
        ctx: &TxContext
    ) {
        let order_hash = keccak256(&order_bytes);
        let order = swap_order_flat::decode(&order_bytes);
        assert!(
            !self.finished_orders.contains(order_hash)
        );
        // make sure user is filling token wanted by order
        assert!(string::from_ascii(type_name::get<T>().into_string())==order.get_to_token());
        // insert order if its first occurrence
        if (!self.pending_fills.contains(order_hash)) {
            self.pending_fills.add(order_hash, order);
        };

        let (remaining, dest_addr, src_nid) = {
            let order = self.pending_fills.borrow_mut<vector<u8>, SwapOrderFlat>(order_hash);
            assert!(
                fill_token.value() <= (order.get_min_receive() as u64)
            );
            let payout = (
                (order.get_amount() as u64) * fill_token.value()
            ) / (order.get_min_receive() as u64);
            order.deduct_min_receive(fill_token.value());
            order.deduct_amount(payout);
            (
                order.get_min_receive(),
                order.get_destination_address(),
                order.get_src_nid()
            )
        };
        if (remaining == 0) {
            self.pending_fills.remove<vector<u8>, SwapOrderFlat>(order_hash);
            self.finished_orders.add(order_hash, true);
        };

        let fill = order_fill::new(
            id,
            order_bytes,
            solveraddress.to_bytes(),
            (fill_token.value() as u128)
        );

        let msg = order_message::new(FILL, fill.encode());
        transfer::public_transfer(
            fill_token,
            suiaddress::from_bytes(dest_addr)
        );
        cluster_connection::send_message(self.get_connection_state_mut(), src_nid, msg.encode());
    }

    entry fun cancel<T: store>(
        self: &mut Storage,
        id: u128,
        ctx: &TxContext
    ) {
        let(msg,dst_nid) ={
            let order = self.orders.borrow<u128, SwapOrder<T>>(id);
            assert!(order.get_creator() == ctx.sender().to_bytes());
            let msg = order_cancel::new(order.encode());
            let order_msg = order_message::new(CANCEL, msg.encode());
            (order_msg,order.get_dst_nid())
        };

        cluster_connection::send_message(self.get_connection_state_mut(), dst_nid, msg.encode());

    }

    // admin functions //

    entry fun set_relayer(self:&mut Storage,cap:&AdminCap,relayer:address){
        self.get_connection_state_mut().set_relayer(relayer);
    }

}
