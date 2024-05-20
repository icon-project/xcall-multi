use rlp::{Decodable, DecoderError, Encodable, Rlp};
use anyhow::Result;
use crate::error::ErrorCode;

pub struct CallMessageWithRollback {
    pub msg_type: u8,
    pub data: Vec<u8>,
    pub rollback: Vec<u8>,
}

impl CallMessageWithRollback {
    pub fn unmarshal(value: &[u8]) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value);
        CallMessageWithRollback::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }

    pub fn unmarshal_from(value: &Vec<u8>) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value as &[u8]);
        CallMessageWithRollback::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }
}

impl Encodable for CallMessageWithRollback {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        s.append(&self.msg_type);
        s.append(&self.data);
        s.append(&self.rollback);
    }
}

impl Decodable for CallMessageWithRollback {
    fn decode(rlp: &rlp::Rlp) -> std::prelude::v1::Result<Self, rlp::DecoderError> {
        if !rlp.is_list() || rlp.item_count()? != 3 {
            return Err(DecoderError::RlpIncorrectListLen);
        }
        let msg_type = rlp.val_at(0)?;
        let data = rlp.val_at(1)?;
        let rollback = rlp.val_at(2)?;

        Ok(CallMessageWithRollback {
            msg_type,
            data,
            rollback,
        })
    }
}
