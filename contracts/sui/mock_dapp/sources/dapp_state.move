module mock_dapp::dapp_state {
    use xcall::xcall_state::IDCap;
    use sui::object::{Self, UID,ID};
    use sui::vec_map::{Self, VecMap};
    use std::vector::{Self};
    use std::string::{Self,String};
    use xcall::execute_ticket::{Self};

    public struct Connection has store,copy,drop{
        source:vector<String>,
        destination:vector<String>,
    }

    public struct ExecuteParams has drop {
        type_args: vector<String>, 
        args: vector<String>,
    }

    public fun create_execute_params(type_args: vector<String>, args: vector<String>): ExecuteParams {
        ExecuteParams{
            type_args:type_args,
            args:args
        }
    }

    public fun get_config_id(config: &DappState): ID {
        config.id.to_inner()
    }

    public fun get_xcall_id(config: &DappState): ID{
        config.xcall_id
    }

    public fun get_connection_source(connection:&Connection):vector<String>{
          connection.source
    }
    public fun get_connection_dest(connection:&Connection):vector<String>{
          connection.destination
    }

    public struct DappState has key{
        id:UID,
        xcall_cap:IDCap,
        xcall_id:ID,
        connections:VecMap<String,Connection>

    }

    public(package) fun new(cap:IDCap, xcall_id: ID, ctx: &mut TxContext):DappState{

        DappState {
            id: object::new(ctx),
            xcall_cap:cap,
            xcall_id:xcall_id,
            connections:vec_map::empty<String,Connection>(),
        }

    }
    public(package) fun share(self:DappState){
         transfer::share_object(self);
    }
    public (package) fun get_xcall_cap(self:&DappState):&IDCap{
        &self.xcall_cap
    }

    public fun id(self:&DappState):ID {
        object::uid_to_inner(&self.id)

    }

    public fun id_str(self:&DappState):String {
        let id=object::uid_to_address(&self.id);
        id.to_string()
    }

    public fun get_connection(self:&DappState,net_id:String):Connection{
        let conn:Connection= *vec_map::get(&self.connections,&net_id);
        conn

    }

    public fun add_connection(self:&mut DappState,net_id:String,source:vector<String>,dest:vector<String>,ctx:&mut TxContext){
        if (vec_map::contains(&self.connections,&net_id)){
            vec_map::remove(&mut self.connections,&net_id);
        };
        vec_map::insert(&mut self.connections,net_id,Connection{source,destination:dest});
    }

}