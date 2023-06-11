use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::Types;


#[cw_serde]
pub struct InstantiateMsg {
    pub x_call: String,
    pub hub_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    Setup {
        _x_call: String,
        _hub_address: String,
    },
    HandleCallMessage {
        _from: String,
        _data: Vec<u8>,
    },
    CrossTransfer {
        to: String,
        amount: u128,
        data: Vec<u8>,
    },
    XCrossTransfer {
        from: String,
        cross_transfer_data: Types::CrossTransfer,
    },
    XCrossTransferRevert {
        from: String,
        cross_transfer_revert_data: Types::CrossTransferRevert,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub enum XCallMsg {
    SendCallMessage {
        to: String,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
    },
}
