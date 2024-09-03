use soroban_sdk::{contracttype, String};

#[contracttype]
pub enum StorageKey {
    XcallAddress,
    Admin,
    Xlm,
    Sn,
    Rollback(u128),
    Connections(String),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Connection {
    pub src_endpoint: String,
    pub dst_endpoint: String,
}

impl Connection {
    pub fn new(src_endpoint: String, dst_endpoint: String) -> Self {
        Connection {
            src_endpoint,
            dst_endpoint,
        }
    }
}
