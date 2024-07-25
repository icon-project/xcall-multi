use soroban_sdk::{contractclient, Address, Bytes, Env, String, Vec};

#[contractclient(name = "DappClient")]
pub trait IDapp {
    fn handle_call_message(
        env: Env,
        sender: Address,
        from: String,
        data: Bytes,
        protocols: Option<Vec<String>>,
    );
}
