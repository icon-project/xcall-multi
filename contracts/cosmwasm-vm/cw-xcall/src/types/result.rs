use super::*;

#[cw_serde]
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

#[cw_serde]
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
            message: reply.unwrap_or_default(),
        }
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn response_code(&self) -> &CallServiceResponseType {
        &self.response_code
    }

    pub fn set_fields(&mut self, sequence_no: u128, response_code: CallServiceResponseType) {
        self.sequence_no.clone_from(&sequence_no);
        self.response_code = response_code;
    }

    pub fn get_message(&self) -> Option<CSMessageRequest> {
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

        stream
            .begin_list(3)
            .append(&self.sequence_no())
            .append(&code)
            .append(&self.message);
    }
}

impl Decodable for CSMessageResult {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let code: u8 = rlp.val_at(1)?;

        Ok(Self {
            sequence_no: rlp.val_at(0)?,
            response_code: CallServiceResponseType::try_from(code)?,
            message: rlp.val_at(2).unwrap_or_default(),
        })
    }
}

impl TryFrom<&Vec<u8>> for CSMessageResult {
    type Error = ContractError;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value as &[u8]);
        Self::decode(&rlp).map_err(|error| ContractError::DecodeFailed {
            error: error.to_string(),
        })
    }
}

impl TryFrom<&[u8]> for CSMessageResult {
    type Error = ContractError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value);
        Self::decode(&rlp).map_err(|error| ContractError::DecodeFailed {
            error: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    /*
    CSMessageResponse
     sn: 1
     code: CSMessageResponse.SUCCESS
     errorMessage: errorMessage
     RLP: C20101

     CSMessageResponse
     sn: 2
     code: CSMessageResponse.FAILURE
     errorMessage: errorMessage
     RLP: C20200
     */

    use common::rlp;

    use super::{CSMessageResult, CallServiceResponseType};

    #[test]
    fn test_cs_message_response_encoding() {
        let cs_response =
            CSMessageResult::new(1, CallServiceResponseType::CallServiceResponseSuccess, None);
        let encoded = rlp::encode(&cs_response);

        assert_eq!("c3010180", hex::encode(encoded));

        let cs_response =
            CSMessageResult::new(2, CallServiceResponseType::CallServiceResponseFailure, None);
        let encoded = rlp::encode(&cs_response);

        assert_eq!("c3020080", hex::encode(encoded));
    }
}
