use soroban_sdk::{Address, Env, String, Vec};
use soroban_xcall_lib::{messages::envelope::Envelope, network_address::NetworkAddress};

use crate::{contract::MockDapp, errors::ContractError, interfaces::interface_xcall::XcallClient};

impl MockDapp {
    pub fn xcall_send_call(
        e: &Env,
        sender: &Address,
        to: &NetworkAddress,
        envelope: &Envelope,
        xcall_address: &Address,
    ) -> u128 {
        let client = XcallClient::new(&e, &xcall_address);
        client.send_call(
            sender,
            &e.current_contract_address(),
            envelope,
            &to.to_string(),
        )
    }

    pub fn xcall_get_fee(
        e: &Env,
        nid: &String,
        rollback: bool,
        sources: Vec<String>,
        xcall_address: &Address,
    ) -> Result<u128, ContractError> {
        let client = XcallClient::new(&e, &xcall_address);
        let fee = client.get_fee(&nid, &rollback, &Some(sources));

        Ok(fee)
    }
}
