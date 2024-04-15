use soroban_sdk::{Bytes, Env, String};

pub(crate) fn send_message(e: &Env, to: String, sn: u128, msg: Bytes) {
    e.events()
        .publish(("CentralizedConnection", "Message", to, sn), msg);
}
