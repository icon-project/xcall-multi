use soroban_sdk::{contracttype, Bytes};

use crate::types::message::{IMessage, MessageType};

pub mod call_message;
pub mod call_message_persisted;
pub mod call_message_rollback;
pub mod cs_message;
pub mod envelope;

use self::{
    call_message::CallMessage, call_message_persisted::CallMessagePersisted,
    call_message_rollback::CallMessageWithRollback,
};

#[contracttype]
pub enum AnyMessage {
    CallMessage(CallMessage),
    CallMessageWithRollback(CallMessageWithRollback),
    CallMessagePersisted(CallMessagePersisted),
}

impl IMessage for AnyMessage {
    fn data(&self) -> Bytes {
        match self {
            AnyMessage::CallMessage(msg) => msg.data(),
            AnyMessage::CallMessagePersisted(msg) => msg.data(),
            AnyMessage::CallMessageWithRollback(msg) => msg.data(),
        }
    }

    fn rollback(&self) -> Option<Bytes> {
        match self {
            AnyMessage::CallMessage(_) => None,
            AnyMessage::CallMessagePersisted(_) => None,
            AnyMessage::CallMessageWithRollback(msg) => msg.rollback(),
        }
    }
}

impl AnyMessage {
    pub fn msg_type(&self) -> MessageType {
        match self {
            AnyMessage::CallMessage(_) => MessageType::CallMessage,
            AnyMessage::CallMessageWithRollback(_) => MessageType::CallMessageWithRollback,
            AnyMessage::CallMessagePersisted(_) => MessageType::CallMessagePersisted,
        }
    }
}
