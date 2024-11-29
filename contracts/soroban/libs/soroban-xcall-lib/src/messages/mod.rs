use soroban_sdk::{contracttype, Bytes};

pub mod call_message;
pub mod call_message_persisted;
pub mod call_message_rollback;
pub mod envelope;
pub mod msg_trait;
pub mod msg_type;

use self::{
    call_message::CallMessage, call_message_persisted::CallMessagePersisted,
    call_message_rollback::CallMessageWithRollback, msg_trait::IMessage, msg_type::MessageType,
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
