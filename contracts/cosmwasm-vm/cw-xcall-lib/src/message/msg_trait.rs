use common::rlp::DecoderError;

use super::msg_type::MessageType;

pub trait IMessage: Clone {
    fn rollback(&self) -> Option<Vec<u8>>;
    fn data(&self) -> Vec<u8>;
    fn msg_type(&self) -> &MessageType;

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError>;
}
