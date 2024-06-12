use super::*;

use request::CSMessageRequest;
use result::CSMessageResult;

#[derive(Clone)]
pub enum CSMessageType {
    CSMessageRequest = 1,
    CSMessageResult,
}

#[derive(Clone)]
pub struct CSMessage {
    pub message_type: CSMessageType,
    pub payload: Vec<u8>,
}

impl CSMessage {
    pub fn new(message_type: CSMessageType, payload: Vec<u8>) -> Self {
        Self {
            message_type,
            payload: payload.to_vec(),
        }
    }

    pub fn message_type(&self) -> &CSMessageType {
        &self.message_type
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(&self.clone()).to_vec()
    }
}

impl Encodable for CSMessage {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        let msg_type: u8 = match self.message_type {
            CSMessageType::CSMessageRequest => 1,
            CSMessageType::CSMessageResult => 2,
        };

        stream.begin_list(2).append(&msg_type).append(&self.payload);
    }
}

impl Decodable for CSMessage {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        if !rlp.is_list() || rlp.item_count()? != 2 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        let msg_type: u8 = rlp.val_at(0)?;

        Ok(Self {
            message_type: match msg_type {
                1 => Ok(CSMessageType::CSMessageRequest),
                2 => Ok(CSMessageType::CSMessageResult),
                _ => Err(rlp::DecoderError::Custom("Invalid type")),
            }?,
            payload: rlp.val_at(1)?,
        })
    }
}

impl From<CSMessageRequest> for CSMessage {
    fn from(value: CSMessageRequest) -> Self {
        Self {
            message_type: CSMessageType::CSMessageRequest,
            payload: rlp::encode(&value).to_vec(),
        }
    }
}

impl From<CSMessageResult> for CSMessage {
    fn from(value: CSMessageResult) -> Self {
        Self {
            message_type: CSMessageType::CSMessageResult,
            payload: rlp::encode(&value).to_vec(),
        }
    }
}
