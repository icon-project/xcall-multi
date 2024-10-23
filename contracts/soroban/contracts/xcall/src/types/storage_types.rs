use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
pub enum StorageKey {
    Admin,
    Config,
    FeeHandler,
    ProtocolFee,
    DefaultConnections(String),
    SuccessfulResponses(u128),
    Sn,
    Rollback(u128),
    ProxyRequest(u128),
    PendingRequests(BytesN<32>),
    PendingResponses(BytesN<32>),
    LastReqId,
    UpgradeAuthority,
}

#[contracttype]
pub struct Config {
    pub network_id: String,
    pub native_token: Address,
}
