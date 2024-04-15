use soroban_sdk::{contract, contractimpl, Env, String, Vec};

#[contract]
pub struct Xcall;

#[contractimpl]
impl Xcall {
    pub fn handle_message(_env: Env, _from_nid: String, _msg: Vec<u32>) {
        unimplemented!()
    }

    pub fn handle_error(_env: Env, _sn: u128) {
        unimplemented!()
    }
}
