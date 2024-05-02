use crate::{
    contract::Xcall,
    event,
    types::{network_address::NetworkAddress, result::CSResponseType},
};
use soroban_sdk::{Address, Bytes, Env, String, Vec};

pub mod dapp {
    soroban_sdk::contractimport!(file = "./wasm/dapp.wasm");
}

impl Xcall {
    pub fn handle_call_message(
        e: &Env,
        address: Address,
        from: &NetworkAddress,
        data: &Bytes,
        protocols: Vec<String>,
    ) {
        let client = dapp::Client::new(&e, &address);
        if protocols.len() > 0 {
            client.handle_call_message(&from.as_string(), data, &Some(protocols))
        } else {
            client.handle_call_message(&from.as_string(), data, &None)
        }
    }

    pub fn try_handle_call_message(
        e: &Env,
        req_id: u128,
        address: Address,
        from: &NetworkAddress,
        data: &Bytes,
        _protocols: Vec<String>,
    ) -> u32 {
        let mut protocols: Option<Vec<String>> = None;
        if _protocols.len() > 0 {
            protocols = Some(_protocols)
        }

        let client = dapp::Client::new(&e, &address);
        let res = client.try_handle_call_message(&from.as_string(), data, &protocols);

        match res {
            Ok(_) => {
                let code = CSResponseType::CSResponseSuccess.into();
                event::call_executed(&e, req_id, code, String::from_str(&e, "success"));
                code
            }
            // TODO: convert error type to string
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
}
