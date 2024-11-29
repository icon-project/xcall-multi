use soroban_sdk::{contractclient, Bytes, Env, String, Vec};

#[contractclient(name = "DappClient")]
pub trait IDapp {
    fn handle_call_message(env: Env, from: String, data: Bytes, protocols: Option<Vec<String>>);
}
