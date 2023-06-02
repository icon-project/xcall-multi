use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

pub mod Types {
    use super::*;

    #[cw_serde]
    pub struct CrossTransfer {
        pub from: String,
        pub to: String,
        pub value: u128,
        pub data: Vec<u8>,
    }

    #[cw_serde]
    pub struct CrossTransferRevert {
        pub from: String,
        pub value: u128,
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub x_call: Addr,
    pub hub_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    Setup {_xCall: Addr, _hubAddress: String},
    HandleCallMessage {_from: Addr, _data: Vec<u8>},
    CrossTransfer {to : Addr, amount: u128, data: Vec<u8>},
    XCrossTransfer {from: Addr, crossTransferData: Types::CrossTransfer},
    XCrossTransferRevert {from: Addr, crossTransferRevertData: Types::CrossTransferRevert},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
   
}
