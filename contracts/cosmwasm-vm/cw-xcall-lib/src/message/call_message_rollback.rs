use common::rlp::{self, Decodable, DecoderError, Encodable, RlpStream};
use cosmwasm_schema::cw_serde;

use super::msg_trait::IMessage;

#[cw_serde]
pub struct CallMessageWithRollback {
    pub data: Vec<u8>,
    pub rollback: Vec<u8>,
}

impl Encodable for CallMessageWithRollback {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream
            .begin_list(2)
            .append(&self.data)
            .append(&self.rollback);
    }
}

impl Decodable for CallMessageWithRollback {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Self {
            data: rlp.val_at(0)?,
            rollback: rlp.val_at(1)?,
        })
    }
}

impl IMessage for CallMessageWithRollback {
    fn rollback(&self) -> Option<Vec<u8>> {
        Some(self.rollback.clone())
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
    fn test_call_message_with_rollback() {
        let msg = CallMessageWithRollback {
            data: vec![1, 2, 3],
            rollback: vec![1, 2, 3],
        };

        let encoded = msg.rlp_bytes().to_vec();
        let decoded = CallMessageWithRollback::decode(&Rlp::new(&encoded)).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg.rollback().unwrap(), msg.rollback);
        assert_eq!(msg.data(), msg.data);
        assert_eq!(msg.to_bytes().unwrap(), encoded)
    }
}
