use soroban_sdk::{contracttype, Address, Bytes, String};

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    CallMessage = 0,
    CallMessageWithRollback = 1,
    CallMessagePersisted = 2,
}

#[contracttype]
pub struct InitializeMsg {
    pub network_id: String,
    pub sender: Address,
    pub native_token: Address,
}

pub trait IMessage {
    fn data(&self) -> Bytes;
    fn rollback(&self) -> Option<Bytes>;
}
