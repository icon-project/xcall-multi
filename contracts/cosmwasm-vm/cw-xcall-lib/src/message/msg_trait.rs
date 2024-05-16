use common::rlp::DecoderError;

pub trait IMessage: Clone {
    fn rollback(&self) -> Option<Vec<u8>>;
    fn data(&self) -> Vec<u8>;

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError>;
}
