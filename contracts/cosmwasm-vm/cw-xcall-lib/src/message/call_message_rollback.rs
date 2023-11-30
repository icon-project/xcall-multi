use common::rlp::{self, Decodable, DecoderError, Encodable, RlpStream};

use super::{msg_trait::IMessage, msg_type::MessageType};

#[derive(Clone, Debug, PartialEq)]
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

    // fn msg_type(&self) -> &MessageType {
    //     &self.msg_type
    // }

    fn to_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        Ok(rlp::encode(self).to_vec())
    }
}
