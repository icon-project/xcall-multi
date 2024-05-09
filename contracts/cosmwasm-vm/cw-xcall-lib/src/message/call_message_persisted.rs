use super::msg_trait::IMessage;
use common::rlp::{self, Decodable, DecoderError, Encodable, RlpStream};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct CallMessagePersisted {
    pub data: Vec<u8>,
}

impl Encodable for CallMessagePersisted {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(1).append(&self.data);
    }
}

impl Decodable for CallMessagePersisted {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Self {
            data: rlp.val_at(0)?,
        })
    }
}

impl IMessage for CallMessagePersisted {
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
    use common::rlp::Rlp;

    use super::*;

    #[test]
    fn test_call_message_persisted() {
        let msg = CallMessagePersisted {
            data: vec![1, 2, 3],
        };

        let encoded = msg.rlp_bytes().to_vec();
        let decoded = CallMessagePersisted::decode(&Rlp::new(&encoded)).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg.rollback(), None);
        assert_eq!(msg.data(), msg.data);
        assert_eq!(msg.to_bytes().unwrap(), encoded)
    }
}
