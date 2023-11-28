use common::rlp::DecoderError;

use self::{
    call_message::CallMessage, call_message_rollback::CallMessageWithRollback, msg_trait::IMessage,
    msg_type::MessageType, call_message_persisted::CallMessagePersisted,
    
};

pub mod call_message;
pub mod call_message_rollback;
pub mod call_message_persisted;
pub mod envelope;
pub mod msg_trait;
pub mod msg_type;

#[derive(Clone, Debug, PartialEq)]
pub enum AnyMessage {
    CallMessage(CallMessage),
    CallMessageWithRollback(CallMessageWithRollback),
    CallMessagePersisted(CallMessagePersisted)
}

impl IMessage for AnyMessage {
    fn rollback(&self) -> Option<Vec<u8>> {
        match self {
            AnyMessage::CallMessage(m) => m.rollback(),
            AnyMessage::CallMessageWithRollback(m) => m.rollback(),
            AnyMessage::CallMessagePersisted(m)=>m.rollback(),
        }
    }

    fn data(&self) -> Vec<u8> {
        match self {
            AnyMessage::CallMessage(m) => m.data(),
            AnyMessage::CallMessageWithRollback(m) => m.data(),
            AnyMessage::CallMessagePersisted(m)=>m.data(),
        }
    }

    fn msg_type(&self) -> &MessageType {
        match self {
            AnyMessage::CallMessage(m) => m.msg_type(),
            AnyMessage::CallMessageWithRollback(m) => m.msg_type(),
            AnyMessage::CallMessagePersisted(m)=>m.msg_type(),
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        match self {
            AnyMessage::CallMessage(m) => m.to_bytes(),
            AnyMessage::CallMessageWithRollback(m) => m.to_bytes(),
            AnyMessage::CallMessagePersisted(m)=> m.to_bytes(),
        }
    }
}
