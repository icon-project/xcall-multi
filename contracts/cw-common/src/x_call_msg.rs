use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
struct NetworkAddress {
    address: String,
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum XCallQuery {
    #[returns(NetworkAddress)]
    GetNetworkAddress {},
}

//TODO: Use the ibc-integration/xcallmsg and xcall contract from ibc
#[cw_serde]
pub enum XCallMsg {
    SendCallMessage {
        to: String,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
    },

    TestHandleCallMessage {
        from: String,
        data: Vec<u8>,
        hub_token: String,
    },
}
