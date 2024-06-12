use rlp::{Decodable, Encodable};

#[derive(Clone, Debug, PartialEq)]
pub enum CallServiceResponseType {
    CallServiceResponseFailure,
    CallServiceResponseSuccess,
}

impl From<CallServiceResponseType> for u8 {
    fn from(val: CallServiceResponseType) -> Self {
        val as u8
    }
}

impl TryFrom<u8> for CallServiceResponseType {
    type Error = rlp::DecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CallServiceResponseType::CallServiceResponseFailure),
            1 => Ok(CallServiceResponseType::CallServiceResponseSuccess),
            _ => Err(rlp::DecoderError::Custom("Invalid type")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSMessageResult {
    sequence_no: u128,
    response_code: CallServiceResponseType,
    message: Vec<u8>,
}

impl CSMessageResult {
    pub fn new(
        sequence_no: u128,
        response_code: CallServiceResponseType,
        reply: Option<Vec<u8>>,
    ) -> Self {
        Self {
            sequence_no,
            response_code,
            message: reply.unwrap_or(vec![]),
        }
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn response_code(&self) -> &CallServiceResponseType {
        &self.response_code
    }

    pub fn message(&self) -> Option<CSMessageResult> {
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
            response_code: CallServiceResponseType::try_from(code)?,
            message: rlp.val_at(2).unwrap_or(vec![]),
        })
    }
}
