module mock_dapp::dapp_state {
    use xcall::xcall_state::IDCap;
    use sui::object::{Self, UID,ID};
    public struct DappState has key{
        id:UID,
        xcall_cap:IDCap,

    }

    public(package) fun new(cap:IDCap,ctx: &mut TxContext):DappState{

        DappState {
            id: object::new(ctx),
            xcall_cap:cap,
        }

    }
    public(package) fun share(self:DappState){
         transfer::share_object(self);
    }
    public (package) fun get_xcall_cap(self:&DappState):&IDCap{
        &self.xcall_cap
    }
}