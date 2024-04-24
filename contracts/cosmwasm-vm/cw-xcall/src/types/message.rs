use super::*;

#[cw_serde]
pub enum CSMessageType {
    CSMessageRequest = 1,
    CSMessageResult,
}

#[cw_serde]
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

impl TryFrom<Binary> for CSMessage {
    type Error = ContractError;

    fn try_from(value: Binary) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(&value);
        Self::decode(&rlp).map_err(|error| ContractError::DecodeFailed {
            error: error.to_string(),
        })
    }
}

impl TryFrom<Vec<u8>> for CSMessage {
    type Error = ContractError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(&value);
        Self::decode(&rlp).map_err(|error| ContractError::DecodeFailed {
            error: error.to_string(),
        })
    }
}

impl From<CSMessage> for Binary {
    fn from(value: CSMessage) -> Self {
        Binary(rlp::encode(&value).to_vec())
    }
}

#[cfg(test)]
mod tests {
    use common::rlp;

    use super::CSMessage;
    /**
    * CSMessage
    type: CSMessage.REQUEST
    data: 7465737431
    RLP: C701857465737431

    CSMessage
    type: CSMessage.RESPONSE
    data: 7465737431
    RLP: C702857465737431
    */

    #[test]
    fn test_csmessage_encoding() {
        let data = hex::decode("7465737431").unwrap();
        let message = CSMessage::new(super::CSMessageType::CSMessageRequest, data.clone());
        let encoded = rlp::encode(&message);

        assert_eq!("c701857465737431", hex::encode(encoded));

        let message = CSMessage::new(crate::types::message::CSMessageType::CSMessageResult, data);
        let encoded = rlp::encode(&message);
        assert_eq!("c702857465737431", hex::encode(encoded));
    }
}
