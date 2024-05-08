use soroban_sdk::{Address, Env};

use crate::{contract::MockDapp, interfaces::interface_xcall::XcallClient};
use xcall::{messages::envelope::Envelope, types::network_address::NetworkAddress};

impl MockDapp {
    pub fn xcall_send_call(
        e: &Env,
        to: &NetworkAddress,
        envelope: &Envelope,
        fee: &u128,
        xcall_address: &Address,
    ) {
        let client = XcallClient::new(&e, &xcall_address);
        client.send_call(envelope, to, &fee, &e.current_contract_address());
    }
}
