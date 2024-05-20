use crate::error::ErrorCode;
use anchor_lang::{prelude::*, solana_program::keccak};
use anyhow::Result;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, PartialEq, Clone)]
pub struct CSMessageRequest {
    pub from: String,
    pub to: String,
    pub sequence_no: u128,
    pub protocols: Vec<String>,
    pub msg_type: u8,
    pub data: Vec<u8>,
}

// impl Encodeable and Decodeable
impl Encodable for CSMessageRequest {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(6);

        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.sequence_no);

        s.begin_list(self.protocols.len());
        for protocol in &self.protocols {
            s.append(protocol);
        }

        s.append(&self.msg_type);
        s.append(&self.data);
    }
}

impl Decodable for CSMessageRequest {
    fn decode(rlp: &Rlp) -> std::prelude::v1::Result<Self, DecoderError> {
        // Verify the expected number of items
        if rlp.item_count()? != 6 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        Ok(CSMessageRequest {
            from: rlp.at(0)?.as_val()?,
            to: rlp.at(1)?.as_val()?,
            sequence_no: rlp.at(2)?.as_val()?,
            protocols: rlp.at(3)?.as_list()?,
            msg_type: rlp.at(4)?.as_val()?,
            data: rlp.at(5)?.data()?.to_vec(),
        })
    }
}

impl CSMessageRequest {
    pub fn new(
        from: String,
        to: String,
        sequence_no: u128,
        msg_type: u8,
        data: Vec<u8>,
        protocols: Vec<String>,
    ) -> Self {
        Self {
            from,
            to,
            sequence_no,
            msg_type,
            data,
            protocols,
        }
    }

    pub fn null() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            sequence_no: 0,
            protocols: vec![String::new()],
            msg_type: u8::MAX,
            data: vec![],
        }
    }

    pub fn unmarshal_from(value: &Vec<u8>) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value as &[u8]);
        CSMessageRequest::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }

    pub fn data_hash(&self) -> Self {
        let hash = keccak::hash(&self.data).as_ref().to_vec();

        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            sequence_no: self.sequence_no,
            protocols: self.protocols.clone(),
            msg_type: self.msg_type,
            data: hash,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(&self.clone()).to_vec()
    }
}
