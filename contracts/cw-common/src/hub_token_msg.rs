use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::network_address::NetworkAddress;

#[cw_serde]
pub struct InstantiateMsg {
    pub x_call: String,
    pub hub_address: String,
}

//TODO: Add network address as a parameter for xcall network address
#[cw_serde]
pub enum ExecuteMsg {
    Setup {
        //TODO: x_call should be of addr type
        x_call: Addr,
        hub_address: NetworkAddress,
    },
    HandleCallMessage {
        from: NetworkAddress,
        data: Vec<u8>,
    },
    CrossTransfer {
        to: NetworkAddress,
        amount: u128,
        data: Vec<u8>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
