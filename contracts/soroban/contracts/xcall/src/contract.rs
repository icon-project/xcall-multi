use soroban_sdk::{contract, contractimpl, Bytes, Env, String};

#[contract]
pub struct Xcall;

#[contractimpl]
impl Xcall {
    pub fn handle_message(_env: Env, _from_nid: String, _msg: Bytes) -> () {}

    pub fn handle_error(_env: Env, _sn: u128) -> () {}
}
