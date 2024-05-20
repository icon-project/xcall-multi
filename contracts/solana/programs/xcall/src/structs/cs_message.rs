use crate::error::ErrorCode;
use anyhow::Result;
use rlp::{Decodable, DecoderError, Encodable, Rlp};

pub enum CSMessageType {
    CSMessageRequest = 1,
    CSMessageResult,
}

impl CSMessageType {
    pub fn to_int(&self) -> u8 {
        match self {
            CSMessageType::CSMessageRequest => 1 as u8,
            CSMessageType::CSMessageResult => 2 as u8,
        }
    }

    pub fn as_type(v: u8) -> Self {
        let a = match v {
            1 => Ok(CSMessageType::CSMessageRequest),
            2 => Ok(CSMessageType::CSMessageResult),
            _ => Err(ErrorCode::InvalidType),
        };
        return a.unwrap();
    }
}

#[derive(Clone)]
pub struct CSMessage {
    pub msg_type: u8,
    pub payload: Vec<u8>,
}

impl Encodable for CSMessage {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(2);
        s.append(&self.msg_type);
        s.append(&self.payload);
    }
}

impl Decodable for CSMessage {
    fn decode(rlp: &rlp::Rlp) -> std::prelude::v1::Result<Self, rlp::DecoderError> {
        if !rlp.is_list() || rlp.item_count()? != 2 {
            return Err(DecoderError::RlpIncorrectListLen);
        }
        let msg_type = rlp.val_at(0)?;
        let payload = rlp.val_at(1)?;

        Ok(CSMessage { msg_type, payload })
    }
}

impl CSMessage {
    pub fn new(msg_type: CSMessageType, payload: Vec<u8>) -> Self {
        Self {
            msg_type: msg_type.to_int(),
            payload: payload,
        }
    }

    pub fn message_type(&self) -> &CSMessageType {
        &self.message_type()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(&self.clone()).to_vec()
    }

    pub fn unmarshal_from(value: &Vec<u8>) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value as &[u8]);
        CSMessage::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }
}
