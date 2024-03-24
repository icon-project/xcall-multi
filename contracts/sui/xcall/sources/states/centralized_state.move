#[allow(unused_field,unused_use,unused_const,unused_mut_parameter,unused_variable,unused_assignment)]
module xcall::centralized_state {
    struct State has store {
        fee:u128,
       
    }

    public fun create():State{
        State {
            fee:0,
        }
    }

    public fun get_fee(self:&State):u128 {
        self.fee
    }
}