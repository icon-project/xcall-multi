use rlp::{Decodable, DecoderError, Encodable, Rlp};

use crate::error::ErrorCode;

pub enum CallServiceResponseType {
    CallServiceResponseFailure = 1,
    CallServiceResponseSuccess,
}

impl CallServiceResponseType {
    pub fn as_int(&self) -> u8 {
        match &self {
            CallServiceResponseType::CallServiceResponseFailure => 1,
            CallServiceResponseType::CallServiceResponseSuccess => 2,
        }
    }

    pub fn from(v : u8) -> Self {
        let a = match v {
            1 => Ok(CallServiceResponseType::CallServiceResponseFailure),
            2 => Ok(CallServiceResponseType::CallServiceResponseSuccess),
            _ => Err(ErrorCode::InvalidType),
        };
        return a.unwrap();
    }
}

pub struct CSMessageResult {
    pub sequence_no: u128,
    pub response_code: u8,
    pub message: Vec<u8>,
}

impl CSMessageResult {
    pub fn new(
        sequence_no: u128,
        response_code: CallServiceResponseType,
        reply: Option<Vec<u8>>,
    ) -> Self {
        let response_code = response_code.as_int();
        Self {
            sequence_no,
            response_code,
            message: reply.unwrap_or(vec![]),
        }
    }

    pub fn unmarshal_from(value: &Vec<u8>) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value as &[u8]);
        CSMessageResult::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }
}

impl Encodable for CSMessageResult {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        s.append(&self.sequence_no);
        s.append(&self.response_code);
        s.append(&self.message);
    }
}

impl Decodable for CSMessageResult {
    fn decode(rlp: &rlp::Rlp) -> std::prelude::v1::Result<Self, rlp::DecoderError> {
        if !rlp.is_list() || rlp.item_count()? != 3 {
            return Err(DecoderError::RlpIncorrectListLen);
        }
        let sequence_no = rlp.val_at(0)?;
        let response_code = rlp.val_at(1)?;
        let message = rlp.val_at(2)?;

        Ok(CSMessageResult {
            sequence_no,
            response_code,
            message,
        })
    }
}
