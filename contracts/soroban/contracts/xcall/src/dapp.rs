use soroban_sdk::{Address, Bytes, Env, String, Vec};
use soroban_xcall_lib::network_address::NetworkAddress;

use crate::{event, interfaces::interface_dapp::DappClient, types::result::CSResponseType};

pub fn handle_call_message(
    e: &Env,
    address: Address,
    from: &NetworkAddress,
    data: &Bytes,
    protocols: Vec<String>,
) {
    let client = DappClient::new(&e, &address);
    if protocols.len() > 0 {
        client.handle_call_message(&from.to_string(), data, &Some(protocols))
    } else {
        client.handle_call_message(&from.to_string(), data, &None)
    }
}

pub fn try_handle_call_message(
    e: &Env,
    req_id: u128,
    address: Address,
    from: &NetworkAddress,
    data: &Bytes,
    _protocols: Vec<String>,
) -> u8 {
    let mut protocols: Option<Vec<String>> = None;
    if _protocols.len() > 0 {
        protocols = Some(_protocols)
    }

    let client = DappClient::new(&e, &address);
    let res = client.try_handle_call_message(&from.to_string(), data, &protocols);

    match res {
        Ok(_) => {
            let code = CSResponseType::CSResponseSuccess.into();
            event::call_executed(&e, req_id, code, String::from_str(&e, "success"));
            code
        }
        Err(err) => match err {
            Ok(_error) => {
                let code = CSResponseType::CSResponseFailure.into();
                event::call_executed(&e, req_id, code, String::from_str(&e, "unknown error"));
                code
            }
            Err(_error) => {
                let code = CSResponseType::CSResponseFailure.into();
                event::call_executed(&e, req_id, code, String::from_str(&e, "unknown error"));
                code
            }
        },
    }
}
