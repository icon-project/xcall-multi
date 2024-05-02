use soroban_sdk::{contracttype, Address, BytesN, String};

use super::network_address::NetId;

#[contracttype]
pub enum StorageKey {
    Admin,
    Config,
    FeeHandler,
    ProtocolFee,
    DefaultConnections(NetId),
    SuccessfulResponses(u128),
    Sn,
    Rollback(u128),
    CallReply,
    ProxyRequest(u128),
    ReplyState,
    PendingRequests(BytesN<32>),
    PendingResponses(BytesN<32>),
    LastReqId,
}

#[contracttype]
pub struct Config {
    pub network_id: String,
    pub native_token: Address,
}
