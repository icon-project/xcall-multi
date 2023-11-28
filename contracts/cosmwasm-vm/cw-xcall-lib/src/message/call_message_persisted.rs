use common::rlp::{self, Decodable, DecoderError, Encodable, RlpStream};

use super::{msg_trait::IMessage, msg_type::MessageType};

#[derive(Clone, Debug, PartialEq)]
pub struct CallMessagePersisted{
    pub msg_type: MessageType,
    pub data: Vec<u8>,
}

impl Encodable for CallMessagePersisted {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream
            .begin_list(2)
            .append(&Into::<u8>::into(self.msg_type.clone()))
            .append(&self.data);
    }
}

impl Decodable for CallMessagePersisted {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let msg_type: u8 = rlp.val_at(0)?;

        Ok(Self {
            msg_type: MessageType::from(msg_type),
            data: rlp.val_at(1)?,
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

    fn msg_type(&self) -> &MessageType {
        &self.msg_type
    }

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        Ok(rlp::encode(self).to_vec())
    }
}
