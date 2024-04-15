use soroban_sdk::{Env, String, Vec};

pub(crate) fn send_message(e: &Env, to: String, sn: u128, msg: Vec<u32>) {
    e.events()
        .publish(("CentralizedConnection", "Message", to, sn), msg);
}
