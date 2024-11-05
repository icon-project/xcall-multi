
/// Module: intents_v1
module intents_v1::main {
    use std::string::{String, Self};
    use sui::linked_table::{ Self};
    use sui::table::{Table, Self};
    use sui::transfer::{ Self };
    use sui::coin::{Coin, Self};
    use sui::bag::{Bag, Self};
    use sui::event::{ Self };
    use intents_v1::order_fill::{OrderFill, Self};
    use intents_v1::order_cancel::{Cancel, Self};
    use intents_v1::order_message::{OrderMessage, Self};
    use intents_v1::swap_order::{Self, SwapOrder};
    use sui::hash::keccak256;
    use sui::address::{Self as suiaddress};
    use intents_v1::cluster_connection::{Self, ConnectionState};
    use sui::hex::{Self};
    use intents_v1::utils::{id_to_hex_string,Self};
    use intents_v1::utils::{get_type_string,address_to_hex_string};
    


    const FILL: u8 = 1; // Constant for Fill message type
    const CANCEL: u8 = 2; // Constant for Cancel message type
    const CURRENT_VERSION: u64 = 1;

    const EAlreadyFinished: u64 = 1;
    const EInvalidFillToken:u64=2;
    const EInvalidPayoutAmount:u64=3;
    const EInvalidMsgType:u64=4;
    const EMsgInvalidSource:u64=5;
    const EInvalidDestination:u64=6;


    public struct AdminCap has key, store {
        id: UID
    }

    public struct Storage has key {
        id: UID,
        version: u64,
        deposit_id: u128, // Deposit ID counter
        nid: String, // Network Identifier
        connection: ConnectionState,
        orders: Table<u128, SwapOrder>, // Mapping of deposit ID to SwapOrder
        finished_orders: Table<vector<u8>, bool>,
        fee: u8,
        fee_handler: address,
        funds:Bag,
    }

    /** Events */
    public struct OrderFilled has copy,drop{
        id:u128,
        src_nid:String,
        fee:u128,
        to_amount:u128,
        solver:String,
    }

    public struct OrderCancelled has copy,drop{
        id:u128, 
        src_nid:String,
        order_bytes:vector<u8>,
    }
    public struct OrderClosed  has copy,drop{
        id:u128,
    }

      public struct Params has drop {
        type_args: vector<String>, 
        args: vector<String>,
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
            finished_orders: table::new(ctx),
            fee: 1,
            fee_handler: ctx.sender(),
            funds:bag::new(ctx),

        };
        transfer::public_transfer(admin, ctx.sender());
        transfer::share_object(storage);

    }

    fun get_next_deposit_id(storage: &mut Storage): u128 {
        let deposit_id = storage.deposit_id + 1;
        storage.deposit_id = deposit_id;
        deposit_id
    }

    fun get_connection_state_mut(self: &mut Storage): &mut ConnectionState {
        &mut self.connection
    }


    /// The resolve_fill function processes the filling of an order for a cross-chain swap.
    /// It verifies the order details, ensures sufficient funds, 
    /// and transfers the specified amount to the solver.
    /// Parameters:
    /// - `self`: Mutable reference to the Storage struct.
    /// - `srcNid`: Source network identifier as a String.
    /// - `fill`: Reference to the OrderFill object.
    /// - `ctx`: Mutable reference to the TxContext.
    fun resolve_fill<T>(
        self: &mut Storage,
        srcNid: String,
        fill: &OrderFill,
        ctx: &mut TxContext
    ) {
       
        let order = self.orders.remove<u128, SwapOrder>(fill.get_id());

        assert!(keccak256(&order.encode()) == keccak256(&fill.get_order_bytes()));
        assert!(order.get_dst_nid() == srcNid);
        let take = self.funds.remove<u128,Coin<T>>(fill.get_id());
        event::emit(OrderClosed { id:fill.get_id() });
        let solver = utils::address_from_str(&fill.get_solver());
        transfer::public_transfer(take, solver);
    }


    /// Processes the cancellation of an order.
    /// Checks if the order is already finished, adds pending fills if necessary,
    /// emits an OrderCancelled event, and sends a message through the cluster connection.
    /// Parameters:
    /// - `self`: Mutable reference to the Storage struct.
    /// - `order_bytes`: Vector of bytes representing the order.
    /// - `ctx`: Mutable reference to the TxContext.
    fun resolve_cancel(
        self: &mut Storage,
        srcNetwork:String,
        order_bytes: vector<u8>,
        ctx: &TxContext
    ) {
        let order_hash = keccak256(&order_bytes);
        let order = swap_order::decode(&order_bytes);

        assert!(order.get_src_nid() == srcNetwork,EMsgInvalidSource);
        assert!(order.get_dst_nid() == self.nid,EInvalidDestination);

        if (self.finished_orders.contains(order_hash)) {
            abort EAlreadyFinished
        };
       
        self.finished_orders.add<vector<u8>, bool>(order_hash, true);

        let orderFill = order_fill::new(
            order.get_id(),
            order_bytes,
            order.get_creator(),
        );

        event::emit(OrderCancelled {
            id:order.get_id(),
            src_nid:order.get_src_nid(),
            order_bytes,
        });

        let msg = order_message::new(FILL, orderFill.encode());
        cluster_connection::send_message(
            self.get_connection_state_mut(),
            order.get_src_nid(),
            msg.encode()
        )
    }
    /// Escrows the specified amount from the user for a cross-chain swap.
    /// Creates a new swap order with the provided details and adds it to the storage.
    /// Parameters:
    /// - `toNid`: Destination network identifier as a String.
    /// - `token`: Coin object representing the token to be swapped.
    /// - `toToken`: Token to be received on the destination network.
    /// - `toAddress`: Address on the destination network for receiving the swapped tokens.
    /// - `minReceive`: Minimum amount to be received in the swap.
    /// - `data`: Additional data for the swap in vector format.
    /// - `ctx`: Mutable reference to the transaction context.
    entry fun swap<T>(
        self: &mut Storage,
        toNid: String,
        token: Coin<T>,
        toToken: String,
        toAddress: String,
        minReceive: u128,
        data: vector<u8>,
        ctx: &TxContext

    ) {
        // Escrows amount from user
        let deposit_id = get_next_deposit_id(self);
        let order = swap_order::new(
            deposit_id,
            id_to_hex_string(&self.get_id()),
            self.nid,
            toNid,
            address_to_hex_string(&ctx.sender()),
            toAddress,
     get_type_string<T>(),
            token.value() as u128,
            toToken,
            minReceive,
            data
        );
        self.funds.add<u128,Coin<T>>(deposit_id, token);

        swap_order::emit(order);
        self.orders.add(deposit_id, order);
        

    }
    /// Receives and processes a message for a cross-chain swap.
    /// Decodes the message type and triggers corresponding actions.
    /// Parameters:
    /// - `self`: Mutable reference to the Storage struct.
    /// - `srcNetwork`: Source network identifier as a String.
    /// - `conn_sn`: Connection serial number as a u128.
    /// - `msg`: Vector of bytes representing the message.
    /// - `ctx`: Mutable reference to the TxContext.
    entry fun receive_message<T>(
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
            resolve_cancel(self, srcNetwork,cancel.get_order_bytes(), ctx);
        }
    }

    /// Fills an order for a cross-chain swap, processes the payment, and handles the fee distribution.
    /// Parameters:
    /// - `id`: The unique identifier of the order.
    /// - `emitter`: The entity emitting the order.
    /// - `src_nid`: Source network identifier.
    /// - `dst_nid`: Destination network identifier.
    /// - `creator`: Creator of the order.
    /// - `destination_address`: Address for receiving swapped tokens.
    /// - `token`: Token to be swapped.
    /// - `amount`: Amount to be filled.
    /// - `to_token`: Token expected to be received.
    /// - `min_receive`: Minimum amount expected to be received.
    /// - `data`: Additional data for the swap.
    /// - `fill_token`: Coin object representing the payment.
    /// - `solveraddress`: Address of the solver handling the order.
    /// - `ctx`: Transaction context for the operation.
    entry fun fill<T,F>(
        self: &mut Storage,
        id: u128,
        emitter:String,                
        src_nid:String,               
        dst_nid: String,             
        creator:String,                
        destination_address:String,    
        token:String,                
        amount:u128,                 
        to_token:String,                 
        min_receive:u128,             
        data:vector<u8>,
        mut fill_token: Coin<F>,
        solveraddress: String,
        ctx: &mut TxContext
    ) {
        let order = swap_order::new(id, emitter, src_nid, dst_nid, creator, destination_address, token, amount, to_token, min_receive, data);

        let order_hash = keccak256(&order.encode());

        assert!(!self.finished_orders.contains<vector<u8>,bool>(order_hash),EAlreadyFinished);

        // make sure user is filling token wanted by order
        assert!(get_type_string<F>()== order.get_to_token(),EInvalidFillToken);
        assert!((fill_token.value() as u128)==order.get_to_amount(),EInvalidPayoutAmount);
        self.finished_orders.add(order_hash, true);
       


        let fee = (fill_token.value() * (self.fee as u64)) / 10000;
        let fee_token = fill_token.split(fee, ctx);

        let fill = order_fill::new(
            id,
            order.encode(),
            solveraddress,
        );
        let msg = order_message::new(FILL, fill.encode());

      

        event::emit(OrderFilled {
            id:order.get_id(),
            src_nid:order.get_src_nid(),
            fee:fee as u128,
            to_amount:fill_token.value() as u128,
            solver:solveraddress,
        });


        transfer::public_transfer(
            fill_token,
            utils::address_from_str(&order.get_destination_address())
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

    /// Cancels an order by emitting a cancellation message and sending it through the cluster connection.
    /// Parameters:
    /// - `self`: Mutable reference to the Storage struct.
    /// - `id`: The unique identifier of the order to be canceled.
    /// - `ctx`: Mutable reference to the TxContext.
    entry fun cancel(
        self: &mut Storage,
        id: u128,
        ctx: &TxContext
    ) {
        let (msg, src_nid, dst_nid) = {
            let order = self.orders.borrow<u128, SwapOrder>(id);
            assert!(
                order.get_creator() == address_to_hex_string(&ctx.sender())
            );
            let msg = order_cancel::new(order.encode());
            let order_msg = order_message::new(CANCEL, msg.encode());
            (
                order_msg,
                order.get_src_nid(),
                order.get_dst_nid(),
            )
        };

        
        if (src_nid == dst_nid) {
            let order = self.orders.borrow<u128, SwapOrder>(id);
            let srcNetwork=self.nid;
            self.resolve_cancel(srcNetwork, order.encode(), ctx)
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

    entry fun set_fee_handler(self:&mut Storage,_cap:&AdminCap,handler:address){
        self.fee_handler=handler;
    }

     entry fun set_fee(self:&mut Storage,_cap:&AdminCap,fee:u8){
        self.fee=fee;
    }

     entry fun get_receive_msg_args(self:&Storage,msg:vector<u8>):Params{
        let msg= order_message::decode(&msg);
        let bytes= if (msg.get_type() == FILL) {
            let fill = order_fill::decode(&msg.get_message());
            fill.get_order_bytes()
        }else if (msg.get_type() == CANCEL) {
            let cancel = order_cancel::decode(&msg.get_message());
            cancel.get_order_bytes()
        } else {
            abort EInvalidMsgType
        };
        
        let order=swap_order::decode(&bytes);
        let token_type=order.get_token();

        let mut type_args:vector<String> = vector::empty();
        type_args.push_back(token_type);

        let mut args:vector<String> = vector::empty();
        args.push_back(utils::id_to_hex_string(&self.get_id()));
        args.push_back(b"srcNid".to_string());  
        args.push_back(b"conn_sn".to_string()); 
        args.push_back(b"msg".to_string());

        Params { type_args, args }


    }

    entry fun get_receipt(self:&Storage,nid:String,conn_sn:u128):bool {
        self.connection.get_receipt(nid, conn_sn)
    }

    entry public fun get_deposit_id(self:&Storage):u128 {
        self.deposit_id
    }

    entry public fun get_order(self:&Storage,id:u128):SwapOrder{
        *self.orders.borrow<u128,SwapOrder>(id)
    }

    entry public fun get_protocol_fee(self:&Storage):u8 {
        self.fee
    }

    

    public fun get_version(self:&Storage):u64{
        self.version

    }
    public fun get_funds(self:&Storage):&Bag {
        &self.funds
    }

    public fun get_id(self:&Storage):ID {
        self.id.to_inner()
    }

    public fun get_relayer(self:&Storage):address{
        self.connection.get_relayer()
    }

    
   

    #[test_only]
    public fun test_init(ctx:&mut TxContext){
        init(ctx);
    }

    #[test_only]
    public fun insert_order<T>(self:&mut Storage,order:&SwapOrder,coin:Coin<T>){
        self.orders.add(order.get_id(), *order);
        self.funds.add(order.get_id(), coin);

    }

}


#[test_only]
module intents_v1::main_tests {
    use sui::test_scenario::{Self, Scenario};
    use sui::coin::{Self, Coin};
    use sui::transfer;
    use sui::bag;
    use sui::address;
    use std::string;
    use intents_v1::main::{Self, Storage, AdminCap};
    use intents_v1::order_fill;
    use intents_v1::order_cancel;
    use intents_v1::order_message;
    use intents_v1::swap_order;
    use intents_v1::utils::id_to_hex_string;
    use intents_v1::main::{insert_order};
    use sui::sui::{SUI as RSUI};
    use intents_v1::utils::{get_type_string,address_to_hex_string};

    // Test coin type
    public struct USDC {}

    public struct SUI {}

    public struct TEST {}

    // Helper function to set up a test scenario
    fun setup_test(admin:address) : Scenario {
        let mut scenario = test_scenario::begin(admin);
        let ctx = test_scenario::ctx(&mut scenario);
        
        // Initialize the module
        main::test_init(ctx);
        
        scenario
    }

    #[test]
    fun test_init() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, admin);
        {
            let storage = test_scenario::take_shared<Storage>(&scenario);
            assert!(storage.get_version() == 1, 0);
            test_scenario::return_shared(storage);
            let admin_cap = test_scenario::take_from_sender<AdminCap>(&scenario);
            test_scenario::return_to_sender(&scenario, admin_cap);
        };
        test_scenario::end(scenario);
    }

    #[test]
    fun test_swap() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"eth"),
                usdc_coin,
                string::utf8(b"ETH"),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            assert!(storage.get_deposit_id() == 1, 0);
            assert!(bag::contains<u128>(storage.get_funds(), 1), 0);
            let deposited= storage.get_funds().borrow<u128,Coin<USDC>>(1);
            assert!(deposited.value()==1000);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    

    #[test]
    fun test_recv_message_fill() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"eth"),
                usdc_coin,
                string::utf8(b"ETH"),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"eth"),
                address_to_hex_string(&@0x1),
                (@0x2).to_string(),
                get_type_string<USDC>(),
                1000,
                string::utf8(b"ETH"),
                900,
                b"test_data"
            );

            let fill = order_fill::new(
                1,
                swap_order::encode(&order),
                (@0x3).to_string(),
               
            );

            let msg = order_message::new(1, order_fill::encode(&fill));

            main::receive_message<USDC>(
                &mut storage,
                string::utf8(b"eth"),
                1,
                order_message::encode(&msg),
                ctx
            );

            // Assert that the order has been processed
            assert!(!bag::contains(storage.get_funds(), 1), 0);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }


    #[test]
    fun test_recv_message_encoding() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let order = swap_order::new(
                3,
                string::utf8(b"0x236100b56f782f11767dccdcb3ce948fd031fcdcf4073bf4a84ee980dd6b7cb0"),
                string::utf8(b"sui"),
                string::utf8(b"0xa869.fuji"),
                string::utf8(b"7b1b1b36d80f6464b0427cd4d4927e1467d53fb4e308304d2a069684d0eae49f"),
                string::utf8(b"0xb89cd0fd9043e5e8144c501b54303b7e8a65be02"),
                string::utf8(b"0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"),
                1000000000,
                string::utf8(b"0x0000000000000000000000000000000000000000"),
                10,
                x""
            );

           let msg_bytes=x"f9019801b90194f9019103b90143f9014003b842307832333631303062353666373832663131373637646363646362336365393438666430333166636463663430373362663461383465653938306464366237636230837375698b3078613836392e66756a69b84037623162316233366438306636343634623034323763643464343932376531343637643533666234653330383330346432613036393638346430656165343966aa307862383963643066643930343365356538313434633530316235343330336237653861363562653032b84a303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030323a3a7375693a3a535549843b9aca00aa3078303030303030303030303030303030303030303030303030303030303030303030303030303030300a80b842307864663861326639346233333236376435633633643237363166396435383264663638663666353261373765613937316666346261363231346566626164653432843b9aca0001";
        
            let msg = order_message::decode(&msg_bytes);
            let fill = order_fill::decode(&msg.get_message());
            let order2= swap_order::decode(&fill.get_order_bytes());
            std::debug::print(&fill);
            std::debug::print(&fill.get_order_bytes());
            std::debug::print(&order.encode());
            std::debug::print(&order);
            std::debug::print(&order2);
             let coin = coin::mint_for_testing<RSUI>(order.get_amount() as u64, ctx);

            insert_order(&mut storage,&order,coin);

            main::receive_message<RSUI>(
                &mut storage,
                string::utf8(b"0xa869.fuji"),
                1,
                msg_bytes,
                ctx
            );

            // Assert that the order has been processed
            assert!(!bag::contains(storage.get_funds(), 1), 0);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

  //   #[test]
    fun test_recv_message_encoding2() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let order = swap_order::new(
                1,
                string::utf8(b"0xd7263bf1148de7bf38dc5fe99b7d2d1d696ffb9b439fa48e4419ba306daa5826"),
                string::utf8(b"sui"),
                string::utf8(b"0xa869.fuji"),
                string::utf8(b"7b1b1b36d80f6464b0427cd4d4927e1467d53fb4e308304d2a069684d0eae49f"),
                string::utf8(b"0xb89cd0fd9043e5e8144c501b54303b7e8a65be02"),
                string::utf8(b"0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"),
                1000000000,
                string::utf8(b"0x0000000000000000000000000000000000000000"),
                10,
                x""
            );

           let msg_bytes=x"f9019801b90194f9019101b90143f9014001b842307864373236336266313134386465376266333864633566653939623764326431643639366666623962343339666134386534343139626133303664616135383236837375698b3078613836392e66756a69b84037623162316233366438306636343634623034323763643464343932376531343637643533666234653330383330346432613036393638346430656165343966aa307862383963643066643930343365356538313434633530316235343330336237653861363562653032b84a303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030323a3a7375693a3a535549843b9aca00aa3078303030303030303030303030303030303030303030303030303030303030303030303030303030300a80b842307864663861326639346233333236376435633633643237363166396435383264663638663666353261373765613937316666346261363231346566626164653432843b9aca0001";

            let msg = order_message::decode(&msg_bytes);
            let fill = order_fill::decode(&msg.get_message());
            let order2= swap_order::decode(&fill.get_order_bytes());
            std::debug::print(&fill);
            std::debug::print(&fill.get_order_bytes());
            std::debug::print(&order.encode());

            std::debug::print(&order2);
             let coin = coin::mint_for_testing<RSUI>(order.get_amount() as u64, ctx);

            insert_order(&mut storage,&order,coin);

            main::receive_message<RSUI>(
                &mut storage,
                string::utf8(b"0xa869.fuji"),
                1,
                msg_bytes,
                ctx
            );

            // Assert that the order has been processed
            assert!(!bag::contains(storage.get_funds(), 1), 0);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = sui::dynamic_field::EFieldDoesNotExist)]
    fun test_recv_message_duplicate_fill() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"eth"),
                usdc_coin,
                string::utf8(b"ETH"),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"eth"),
                address_to_hex_string(&@0x1),
                (@0x2).to_string(),
                get_type_string<USDC>(),
                1000,
                b"ETH".to_string(),
                900,
                b"test_data"
            );

            let fill = order_fill::new(
                1,
                swap_order::encode(&order),
                (@0x3).to_string(),
               
            );

            let msg = order_message::new(1, order_fill::encode(&fill));

            main::receive_message<USDC>(
                &mut storage,
                string::utf8(b"eth"),
                1,
                order_message::encode(&msg),
                ctx
            );

            // Attempt to process the same fill again, should fail
            main::receive_message<USDC>(
                &mut storage,
                string::utf8(b"eth"),
                2,
                order_message::encode(&msg),
                ctx
            );

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    fun test_complete_fill() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                get_type_string<SUI>(),
                @0x2.to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"sui"),
                address_to_hex_string(&@0x1),
                @0x2.to_string(),
                get_type_string<USDC>(),
                1000,
                get_type_string<SUI>(),
                900,
                b"test_data"
            );

            let fill_coin = coin::mint_for_testing<SUI>(900, ctx);
            main::fill<USDC,SUI>(
                &mut storage,
                order.get_id(),
                order.get_emitter(),
                order.get_src_nid(),
                order.get_dst_nid(),
                order.get_creator(),
                order.get_destination_address(),
                order.get_token(),
                order.get_amount(),
                order.get_to_token(),
                order.get_to_amount(),
                *order.get_data(),
                fill_coin,
                (@0x3).to_string(),
                ctx
            );

            // Assert that the order has been filled
            assert!(!bag::contains<u128>(storage.get_funds(), 1), 0);
            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }


    #[test]
    #[expected_failure(abort_code = 3)]
    fun test_partial_fill() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                get_type_string<SUI>(),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"sui"),
                (@0x1).to_string(),
                (@0x2).to_string(),
                get_type_string<USDC>(),
                1000,
                get_type_string<SUI>(),
                900,
                b"test_data"
            );

            let fill_coin = coin::mint_for_testing<SUI>(800, ctx);
            main::fill<USDC,SUI>(
                &mut storage,
               order.get_id(),
                order.get_emitter(),
                order.get_src_nid(),
                order.get_dst_nid(),
                order.get_creator(),
                order.get_destination_address(),
                order.get_token(),
                order.get_amount(),
                order.get_to_token(),
                order.get_to_amount(),
                *order.get_data(),
                fill_coin,
                (@0x3).to_string(),
                ctx
            );

            // Assert that the order has been filled
            assert!(bag::contains<u128>(storage.get_funds(), 1), 0);

            let coin= storage.get_funds().borrow<u128,Coin<USDC>>(1);
            assert!(coin.value()==112);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = 1)]
    fun test_fill_already_finished() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                get_type_string<SUI>(),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"sui"),
                address_to_hex_string(&(@0x1)),
                (@0x2).to_string(),
                get_type_string<USDC>(),
                1000,
                get_type_string<SUI>(),
                900,
                b"test_data"
            );

            let fill_coin1 = coin::mint_for_testing<SUI>(900, ctx);
            main::fill<USDC,SUI>(
                &mut storage,
               order.get_id(),
                order.get_emitter(),
                order.get_src_nid(),
                order.get_dst_nid(),
                order.get_creator(),
                order.get_destination_address(),
                order.get_token(),
                order.get_amount(),
                order.get_to_token(),
                order.get_to_amount(),
                *order.get_data(),
                fill_coin1,
                @0x3.to_string(),
                ctx
            );

            // Attempt to fill the same order again, should fail
            let fill_coin2 = coin::mint_for_testing<SUI>(900, ctx);
            main::fill<USDC,SUI>(
                &mut storage,
                order.get_id(),
                order.get_emitter(),
                order.get_src_nid(),
                order.get_dst_nid(),
                order.get_creator(),
                order.get_destination_address(),
                order.get_token(),
                order.get_amount(),
                order.get_to_token(),
                order.get_to_amount(),
                *order.get_data(),
                fill_coin2,
                @0x3.to_string(),
                ctx
            );

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = 2)]
    fun test_fill_invalid_token_finished() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                get_type_string<SUI>(),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"sui"),
                (@0x1).to_string(),
                (@0x2).to_string(),
                get_type_string<USDC>(),
                1000,
                get_type_string<SUI>(),
                900,
                b"test_data"
            );

            let fill_coin1 = coin::mint_for_testing<TEST>(900, ctx);
            main::fill<USDC,TEST>(
                &mut storage,
               order.get_id(),
                order.get_emitter(),
                order.get_src_nid(),
                order.get_dst_nid(),
                order.get_creator(),
                order.get_destination_address(),
                order.get_token(),
                order.get_amount(),
                order.get_to_token(),
                order.get_to_amount(),
                *order.get_data(),
                fill_coin1,
                @0x3.to_string(),
                ctx
            );

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

     #[test]
    #[expected_failure(abort_code = 3)]
    fun test_fill_invalid_payout_amount() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                get_type_string<SUI>(),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            let order = swap_order::new(
                1,
                id_to_hex_string(&storage.get_id()),
                string::utf8(b"sui"),
                string::utf8(b"sui"),
                (@0x1).to_string(),
                (@0x2).to_string(),
                get_type_string<USDC>(),
                1000,
                get_type_string<SUI>(),
                900,
                b"test_data"
            );

            let fill_coin1 = coin::mint_for_testing<SUI>(1100, ctx);
            main::fill<USDC,SUI>(
                &mut storage,
               order.get_id(),
                order.get_emitter(),
                order.get_src_nid(),
                order.get_dst_nid(),
                order.get_creator(),
                order.get_destination_address(),
                order.get_token(),
                order.get_amount(),
                order.get_to_token(),
                order.get_to_amount(),
                *order.get_data(),
                fill_coin1,
                @0x3.to_string(),
                ctx
            );

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    fun test_cancel() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                get_type_string<SUI>(),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            main::cancel(&mut storage, 1, ctx);

            // Assert that the order has been cancelled
            assert!(!bag::contains(storage.get_funds(), 1), 0);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    #[expected_failure]
    fun test_cancel_by_non_creator() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);
            
            let usdc_coin = coin::mint_for_testing<USDC>(1000, ctx);
            main::swap<USDC>(
                &mut storage,
                string::utf8(b"sui"),
                usdc_coin,
                string::utf8(b"ETH"),
                (@0x2).to_string(),
                900,
                b"test_data",
                ctx
            );

            test_scenario::next_tx(&mut scenario, @0x2);
            let ctx = test_scenario::ctx(&mut scenario);

            // Attempt to cancel by a different address, should fail
            main::cancel(&mut storage, 1, ctx);

            test_scenario::return_shared(storage);
        };
        test_scenario::end(scenario);
    }

    #[test]
    fun test_set_relayer() {
        let admin=@0x1;
        let mut scenario = setup_test(admin);
        test_scenario::next_tx(&mut scenario, @0x1);
        {
            let mut storage = test_scenario::take_shared<Storage>(&scenario);
            let admin_cap = test_scenario::take_from_sender<AdminCap>(&scenario);
            main::set_relayer(&mut storage, &admin_cap, @0x4);
            assert!(storage.get_relayer()==@0x4);
            test_scenario::return_shared(storage);
            test_scenario::return_to_sender(&scenario, admin_cap);
        };
        test_scenario::end(scenario);
    }
}