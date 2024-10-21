use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw_xcall_lib::network_address::NetId;

#[cw_serde]
pub enum ExecuteMsg {
    SetAdmin {
        address: Addr,
    },

    SetRelayer {
        address: Addr,
    },

    SetValidators {
        validators: Vec<String>,
        threshold: u8,
    },

    SetSignatureThreshold {
        threshold: u8,
    },

    SetFee {
        network_id: NetId,
        message_fee: u128,
        response_fee: u128,
    },

    ClaimFees {},

    SendMessage {
        to: NetId,
        sn: i64,
        msg: Vec<u8>,
    },

    RecvMessage {
        src_network: NetId,
        conn_sn: u128,
        msg: String,
    },

    RecvMessageWithSignatures {
        src_network: NetId,
        conn_sn: u128,
        msg: String,
        signatures: Vec<Vec<u8>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(u64)]
    GetFee { nid: NetId, response: bool },

    #[returns(bool)]
    GetReceipt { src_network: NetId, conn_sn: u128 },

    #[returns(Addr)]
    GetAdmin {},

    #[returns(Addr)]
    GetRelayer {},

    #[returns(Vec<String>)]
    GetValidators {},

    #[returns(u16)]
    GetSignatureThreshold {},
}

#[cw_serde]
pub struct MigrateMsg {}
