use soroban_sdk::{Address, Bytes, Env, String};

use crate::{
    contract::Xcall, errors::ContractError, interfaces::interface_connection::ConnectionClient,
};

impl Xcall {
    pub fn query_connection_fee(
        e: &Env,
        nid: &String,
        response: bool,
        connection: &String,
    ) -> Result<u128, ContractError> {
        let client = ConnectionClient::new(&e, &Address::from_string(&connection));
        let fee = client.get_fee(&nid, &response);

        Ok(fee)
    }

    pub fn call_connection_send_message(
        e: &Env,
        tx_origin: &Address,
        connection: &String,
        nid: &String,
        sn: i64,
        msg: &Bytes,
    ) -> Result<(), ContractError> {
        let client = ConnectionClient::new(&e, &Address::from_string(&connection));
        client.send_message(&tx_origin, &nid, &sn, &msg);

        Ok(())
    }
}
