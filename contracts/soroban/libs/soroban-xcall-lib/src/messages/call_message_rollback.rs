use soroban_sdk::{contracttype, Bytes};

use super::msg_trait::IMessage;

#[contracttype]
pub struct CallMessageWithRollback {
    pub data: Bytes,
    pub rollback: Bytes,
}

impl IMessage for CallMessageWithRollback {
    fn data(&self) -> Bytes {
        self.data.clone()
    }

    fn rollback(&self) -> Option<Bytes> {
        Some(self.rollback.clone())
    }
}
