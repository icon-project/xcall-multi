use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use common :: Types;

#[cw_serde]
pub struct InstantiateMsg {
    pub xCall: Addr,
    pub hubAddress: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    setup {_xCall: Addr, _hubAddress: String},
    handleCallMessage {_from: Addr, _data: String},
    crossTransfer {to : Addr, amount: u128, data: Bytes},
    xCrossTransfer {from: Addr, crossTransferData: Types::CrossTransfer},
    xCrossTransferRevert {from: Addr, crossTransferRevertData: Types::CrossTransferRevert},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
   
}
