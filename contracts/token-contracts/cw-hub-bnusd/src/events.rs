use cosmwasm_std::{Addr, Event};
use cw_common::network_address::NetworkAddress;
use debug_print::debug_println;

pub fn emit_cross_transfer_event(
    name: String,
    from: NetworkAddress,
    to: NetworkAddress,
    amount: u128,
    data: Vec<u8>,
) -> Event {
    Event::new(name)
        .add_attribute("from", from.to_string())
        .add_attribute("to", to.to_string())
        .add_attribute("value", amount.to_string())
        .add_attribute("data", hex_encode(data))
}

pub fn emit_cross_transfer_revert_event(name: String, from: Addr, amount: u128) -> Event {
    Event::new(name)
        .add_attribute("from", from.to_string())
        .add_attribute("value", amount.to_string())
}

fn hex_encode(data: Vec<u8>) -> String {
    debug_println!("this is {:?}", data);
    if data.is_empty() {
        debug_println!("this is empty");
        "null".to_string()
    } else {
        let data = hex::encode(data);
        debug_println!("this is not empty, {}", data);
        data
    }
}
