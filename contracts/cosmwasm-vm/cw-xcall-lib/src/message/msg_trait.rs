use common::rlp::DecoderError;

use super::msg_type::MessageType;

pub trait IMessage: Clone {
    fn rollback(&self) -> Option<Vec<u8>>;
    fn data(&self) -> Vec<u8>;
    fn msg_type(&self) -> &MessageType;
    // fn from_bytes(bytes: Vec<u8>) -> Result<Self, rlp::DecoderError> {
    //     let rlp = rlp::Rlp::new(&bytes);
    //     let msg = Self::decode(&rlp)?;

    //     Ok(msg)
    // }

    // fn to_bytes(&self) -> Vec<u8> {
    //     rlp::encode(self).to_vec()
    // }
    fn should_persist(&self) -> bool;
    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError>;
    //fn from_bytes(bytes:Vec<u8>)->Result<Self,DecoderError>;
}
