use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
struct NetworkAddress {
    address: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum XCallQuery {
    //later in case of multiplre bridges
    //     #[returns(NetworkAddress)]
    //     GetNetworkAddress {
    //         protocol: String,
    //         network_id: String,
    //      },
    // }
    #[returns(NetworkAddress)]
    GetNetworkAddress {},
}

#[cw_serde]
pub enum XCallMsg {
    SendCallMessage {
        to: String,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
    },
}
