use soroban_sdk::{Address, Bytes, Env, String};

use crate::{
    contract::Xcall, errors::ContractError,
    interfaces::interface_centralized_connection::CentralizedConnectionClient,
};

impl Xcall {
    pub fn query_connection_fee(
        e: &Env,
        nid: &String,
        response: bool,
        address: &String,
    ) -> Result<u128, ContractError> {
        let client = CentralizedConnectionClient::new(&e, &Address::from_string(&address));
        let fee = client.get_fee(&nid, &response);

        Ok(fee)
    }

    pub fn call_connection_send_message(
        e: &Env,
        address: &String,
        amount: u128,
        nid: &String,
        sn: i64,
        msg: &Bytes,
    ) -> Result<(), ContractError> {
        Self::ensure_data_size(msg.len() as usize)?;
        let client = CentralizedConnectionClient::new(&e, &Address::from_string(&address));
        client.send_message(&amount, &nid, &sn, &msg);

        Ok(())
    }
}
