use super::*;

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
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

    fn to_bytes(&self) -> Result<Vec<u8>, rlp::DecoderError> {
        Ok(rlp::encode(self).to_vec())
    }
}
