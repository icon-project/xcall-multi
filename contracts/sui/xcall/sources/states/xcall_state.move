module xcall::xcall_state {
     use sui::object::{Self, UID,ID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::linked_table::{Self, LinkedTable};
    use sui::types as sui_types;
    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option};
   
    use xcall::network_address::{Self,NetworkAddress};
    use xcall::envelope::{Self,XCallEnvelope};
    use xcall::connection_out::{Self,register};
    use xcall::message_request::{Self};
    use xcall::cs_message::{Self};
    use xcall::rollback_data::{Self,RollbackData};
    use sui::bag::{Bag, Self};
    use sui::table::{Table,Self};
    use sui::package::{Self,Publisher};
  
    use sui::vec_map::{Self, VecMap};
    use sui::versioned::{Self, Versioned};

    friend xcall::main;

     struct IDCap has key,store {
        id:UID,
        xcall_id:ID,
    }
    struct PackageCap has store {
        package_id:String,
    }
     struct AdminCap has key {
        id: UID
    }


     struct Storage has key {
        id: UID,
        version:u64,
        admin:ID,
        requests:LinkedTable<u128, vector<u8>>,
        sequence_no:u128,
        protocol_fee:u128,
        protocol_fee_handler:address,
        connection_states:Bag,
        rollbacks:Table<u128,RollbackData>,
        connections:VecMap<String,String>,
    }

    public(friend) fun create_admin_cap(ctx: &mut TxContext):AdminCap {
         let admin = AdminCap {
            id: object::new(ctx),
        };
        admin
    }

    public(friend) fun create_id_cap(storage:&Storage,ctx: &mut TxContext):IDCap {
          IDCap {
            id: object::new(ctx),
            xcall_id:object::id(storage)

        }

    }

    public(friend) fun create_storage(version:u64,admin:&AdminCap, ctx: &mut TxContext):Storage {
         let storage = Storage {
            id: object::new(ctx),
            version:version,
            admin:object::id(admin),
            requests:linked_table::new<u128, vector<u8>>(ctx),
            sequence_no:0,
            protocol_fee:0,
            protocol_fee_handler: tx_context::sender(ctx),
            connection_states:bag::new(ctx),
            rollbacks:table::new<u128,RollbackData>(ctx),
            connections:vec_map::empty<String,String>(),
        };
        storage
    }

    public(friend) fun set_version(self:&mut Storage,version:u64){
            self.version=version;
    }

     public(friend) fun set_connection(self:&mut Storage,net_id:String,package_id:String){
            vec_map::insert(&mut self.connections,net_id,package_id);
    }

    public(friend) fun add_rollback(self:&mut Storage,sequence_no:u128,rollback:RollbackData){
         table::add(&mut self.rollbacks,sequence_no,rollback);
    }

    public fun get_version(self:&Storage):u64{
        self.version
    }

    public fun get_admin(self:&Storage):ID {
        self.admin
    }

    public fun get_connection(self:&Storage,nid:String):String{
        *vec_map::get(&self.connections,&nid)
    }

    public fun get_connection_states(self:&mut Storage):&mut Bag{
        &mut self.connection_states
    }

    public(friend) fun get_next_sequence(self:&mut Storage):u128 {
        let sn=self.sequence_no+1;
        self.sequence_no=sn;
        sn
    }


    public(friend) fun set_protocol_fee(self:&mut Storage,fee:u128){
        self.protocol_fee=fee;
    }

    public(friend) fun set_protocol_fee_handler(self:&mut Storage,fee_handler:address){
        self.protocol_fee_handler=fee_handler;
    }

    public(friend) fun transfer_admin_cap(admin:AdminCap,ctx: &mut TxContext){
        transfer::transfer(admin, tx_context::sender(ctx));
    }

    public(friend) fun share(self:Storage){
         transfer::share_object(self);
    }

}