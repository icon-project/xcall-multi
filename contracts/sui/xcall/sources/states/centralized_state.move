module xcall::centralized_state {
    struct State has store {
        fee:u64,
       
    }

    public fun create():State{
        State {
            fee:0,
        }
    }
}