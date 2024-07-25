use soroban_sdk::{contracttype, Bytes};

use super::msg_trait::IMessage;

#[contracttype]
pub struct CallMessage {
    pub data: Bytes,
}

impl IMessage for CallMessage {
    fn data(&self) -> Bytes {
        self.data.clone()
    }

    fn rollback(&self) -> Option<Bytes> {
        None
    }
}
