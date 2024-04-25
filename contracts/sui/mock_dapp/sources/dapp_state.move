module mock_dapp::dapp_state {
    use xcall::xcall_state::IDCap;
    use sui::object::{Self, UID,ID};
   use sui::vec_map::{Self, VecMap};
    use std::vector::{Self};
    use std::string::{String};
    use xcall::execute_ticket::{Self};

    public struct Connection has store,copy,drop{
        source:String,
        destination:String,
    }

    public fun get_connection_source(connection:&Connection):vector<String>{
          let mut sources=vector::empty<String>();
          sources.push_back(connection.source);
          sources
    }
    public fun get_connection_dest(connection:&Connection):vector<String>{
          let mut sources=vector::empty<String>();
          sources.push_back(connection.destination);
          sources
    }

    public struct DappState has key{
        id:UID,
        xcall_cap:IDCap,
        connections:VecMap<String,Connection>

    }

    public(package) fun new(cap:IDCap,ctx: &mut TxContext):DappState{

        DappState {
            id: object::new(ctx),
            xcall_cap:cap,
            connections:vec_map::empty<String,Connection>(),
        }

    }
    public(package) fun share(self:DappState){
         transfer::share_object(self);
    }
    public (package) fun get_xcall_cap(self:&DappState):&IDCap{
        &self.xcall_cap
    }

    public fun get_connection(self:&DappState,net_id:String):Connection{
        let conn:Connection= *vec_map::get(&self.connections,&net_id);
        conn

    }

    public fun add_connection(self:&mut DappState,net_id:String,source:String,dest:String){
        vec_map::insert(&mut self.connections,net_id,Connection{source,destination:dest});
    }

}