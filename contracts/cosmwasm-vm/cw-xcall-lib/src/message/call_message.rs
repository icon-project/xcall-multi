use common::rlp::{self, Decodable, DecoderError, Encodable, RlpStream};
use cosmwasm_schema::cw_serde;

use super::msg_trait::IMessage;

#[cw_serde]
pub struct CallMessage {
    pub data: Vec<u8>,
}

impl Encodable for CallMessage {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(1).append(&self.data);
    }
}

impl Decodable for CallMessage {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Self {
            data: rlp.val_at(0)?,
        })
    }
}

impl IMessage for CallMessage {
    fn rollback(&self) -> Option<Vec<u8>> {
        None
    }

    fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        Ok(rlp::encode(self).to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rlp::Rlp;

    #[test]
    fn test_call_message() {
        let msg = CallMessage {
            data: vec![1, 2, 3],
        };

        let encoded = msg.rlp_bytes().to_vec();
        let decoded = CallMessage::decode(&Rlp::new(&encoded)).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg.rollback(), None);
        assert_eq!(msg.data(), msg.data);
        assert_eq!(msg.to_bytes().unwrap(), encoded)
    }
}
