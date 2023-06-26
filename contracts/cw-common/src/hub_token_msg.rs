use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub x_call: String,
    pub hub_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Setup {
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
