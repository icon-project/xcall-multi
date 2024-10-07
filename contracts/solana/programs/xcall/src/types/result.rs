use super::*;

use crate::error::*;
use request::CSMessageRequest;

#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum CSResponseType {
    CSResponseFailure,
    CSResponseSuccess,
}

impl From<CSResponseType> for u8 {
    fn from(val: CSResponseType) -> Self {
        val as u8
    }
}

impl TryFrom<u8> for CSResponseType {
    type Error = rlp::DecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CSResponseType::CSResponseFailure),
            1 => Ok(CSResponseType::CSResponseSuccess),
            _ => Err(rlp::DecoderError::Custom("Invalid type")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct CSMessageResult {
    sequence_no: u128,
    response_code: CSResponseType,
    message: Vec<u8>,
}

impl CSMessageResult {
    pub fn new(sequence_no: u128, response_code: CSResponseType, reply: Option<Vec<u8>>) -> Self {
        Self {
            sequence_no,
            response_code,
            message: reply.unwrap_or(vec![]),
        }
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn response_code(&self) -> &CSResponseType {
        &self.response_code
    }

    pub fn message(&self) -> Option<CSMessageRequest> {
        if self.message.is_empty() {
            return None;
        }
        rlp::decode(&self.message).ok()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(&self.clone()).to_vec()
    }
}

impl Encodable for CSMessageResult {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        let code: u8 = self.response_code.clone().into();
        stream.begin_list(3);
        stream.append(&self.sequence_no());
        stream.append(&code);
        stream.append(&self.message);
    }
}

impl Decodable for CSMessageResult {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let code: u8 = rlp.val_at(1)?;

        Ok(Self {
            sequence_no: rlp.val_at(0)?,
            response_code: CSResponseType::try_from(code)?,
            message: rlp.val_at(2).unwrap_or(vec![]),
        })
    }
}

impl TryFrom<&Vec<u8>> for CSMessageResult {
    type Error = XcallError;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value as &[u8]);
        Self::decode(&rlp).map_err(|_error| XcallError::DecodeFailed)
    }
}

impl TryFrom<&[u8]> for CSMessageResult {
    type Error = XcallError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value);
        Self::decode(&rlp).map_err(|_error| XcallError::DecodeFailed)
    }
}
