use common::rlp::{self, Decodable, DecoderError, Encodable, RlpStream};

use super::{msg_trait::IMessage, msg_type::MessageType};

#[derive(Clone, Debug, PartialEq)]
pub struct CallMessageWithRollback {
    pub msg_type: MessageType,
    pub data: Vec<u8>,
    pub rollback: Vec<u8>,
}

impl Encodable for CallMessageWithRollback {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream
            .begin_list(3)
            .append(&Into::<u8>::into(self.msg_type.clone()))
            .append(&self.data)
            .append(&self.rollback);
    }
}

impl Decodable for CallMessageWithRollback {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let msg_type: u8 = rlp.val_at(0)?;

        Ok(Self {
            msg_type: MessageType::from(msg_type),
            data: rlp.val_at(1)?,
            rollback: rlp.val_at(2)?,
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

    fn msg_type(&self) -> &MessageType {
        &self.msg_type
    }

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        Ok(rlp::encode(self).to_vec())
    }
}
