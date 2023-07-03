use cosmwasm_schema::{cw_serde, QueryResponses};

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
        x_call: String,
        hub_address: String,
    },
    HandleCallMessage {
        from: String,
        data: Vec<u8>,
    },
    CrossTransfer {
        to: String,
        amount: u128,
        data: Vec<u8>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
