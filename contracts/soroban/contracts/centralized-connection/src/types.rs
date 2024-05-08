use soroban_sdk::{contracttype, Address, Bytes, String};

#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Xcall,
    Admin,
    Xlm,
    ConnSn,
    MessageFee(String),
    ResponseFee(String),
    Receipts(String, u128),
}

#[contracttype]
pub struct InitializeMsg {
    pub relayer: Address,
    pub native_token: Address,
    pub xcall_address: Address,
}

#[contracttype]
pub struct SendMsgEvent {
    pub target_network: String,
    pub conn_sn: u128,
    pub msg: Bytes,
}
