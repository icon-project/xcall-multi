use common::rlp::DecoderError;
use cosmwasm_schema::cw_serde;

use self::{
    call_message::CallMessage, call_message_persisted::CallMessagePersisted,
    call_message_rollback::CallMessageWithRollback, msg_trait::IMessage, msg_type::MessageType,
};

pub mod call_message;
pub mod call_message_persisted;
pub mod call_message_rollback;
pub mod envelope;
pub mod msg_trait;
pub mod msg_type;
#[cw_serde]
pub enum AnyMessage {
    CallMessage(CallMessage),
    CallMessageWithRollback(CallMessageWithRollback),
    CallMessagePersisted(CallMessagePersisted),
}

impl IMessage for AnyMessage {
    fn rollback(&self) -> Option<Vec<u8>> {
        match self {
            AnyMessage::CallMessage(m) => m.rollback(),
            AnyMessage::CallMessageWithRollback(m) => m.rollback(),
            AnyMessage::CallMessagePersisted(m) => m.rollback(),
        }
    }

    fn data(&self) -> Vec<u8> {
        match self {
            AnyMessage::CallMessage(m) => m.data(),
            AnyMessage::CallMessageWithRollback(m) => m.data(),
            AnyMessage::CallMessagePersisted(m) => m.data(),
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        match self {
            AnyMessage::CallMessage(m) => m.to_bytes(),
            AnyMessage::CallMessageWithRollback(m) => m.to_bytes(),
            AnyMessage::CallMessagePersisted(m) => m.to_bytes(),
        }
    }
}

impl AnyMessage {
    pub fn msg_type(&self) -> &MessageType {
        match self {
            AnyMessage::CallMessage(_m) => &MessageType::CallMessage,
            AnyMessage::CallMessageWithRollback(_m) => &MessageType::CallMessageWithRollback,
            AnyMessage::CallMessagePersisted(_m) => &MessageType::CallMessagePersisted,
        }
    }
}
