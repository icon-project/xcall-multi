module mock_dapp::mock_dapp {
    use xcall::main::{Self as xcall};
    use xcall::xcall_state::{Storage as XCallState};
    use mock_dapp::dapp_state::{Self,DappState};


public struct REGISTER_WITNESS has store,drop {}
public struct WitnessCarrier has key { id: UID, witness: REGISTER_WITNESS }


    fun init(ctx: &mut TxContext) {


         transfer::transfer(
            WitnessCarrier { id: object::new(ctx), witness:REGISTER_WITNESS{} },
            ctx.sender()
        );
       
    }

    entry fun register_xcall(xcall:&XCallState,carrier:WitnessCarrier,ctx:&mut TxContext){
        let w= get_witness(carrier);
        let cap= xcall::register_dapp(xcall,w,ctx);
        let state=dapp_state::new(cap,ctx);
        dapp_state::share(state);

    }

     fun get_witness(carrier: WitnessCarrier): REGISTER_WITNESS {
        let WitnessCarrier { id, witness } = carrier;
        id.delete();
        witness
    }

    entry fun execute_call(state:&mut DappState,xcall:&mut XCallState,request_id:u128,data:vector<u8>,ctx:&mut TxContext){
        let ticket= xcall::execute_call(xcall,dapp_state::get_xcall_cap(state),request_id,data,ctx);
        xcall::execute_call_result(xcall,ticket,true);


    }

    

    


}