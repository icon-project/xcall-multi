extern crate rlp;

use crate::error::ErrorCode;
use anyhow::Result;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XCallEnvelope {
    pub msg_type: u8,
    pub message: Vec<u8>,
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

impl XCallEnvelope {
    pub fn unmarshal(value: &[u8]) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value);
        XCallEnvelope::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }

    pub fn unmarshal_from(value: &Vec<u8>) -> Result<Self, ErrorCode> {
        let rlp = Rlp::new(value as &[u8]);
        XCallEnvelope::decode(&rlp).map_err(|_error| ErrorCode::XCallEnvelopeDecodeError)
    }
}

impl Encodable for XCallEnvelope {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);
        s.append(&self.msg_type);
        s.append(&self.message);

        s.begin_list(self.sources.len());
        for src in &self.sources {
            s.append(src);
        }

        s.begin_list(self.destinations.len());
        for dst in &self.destinations {
            s.append(dst);
        }
    }
}

impl Decodable for XCallEnvelope {
    fn decode(rlp: &Rlp) -> std::prelude::v1::Result<Self, DecoderError> {
        if !rlp.is_list() || rlp.item_count()? != 4 {
            return Err(DecoderError::RlpIncorrectListLen);
        }
        let msg_type = rlp.val_at(0)?;
        let message = rlp.val_at(1)?;

        let sources = rlp.at(2)?.as_list()?;
        let destinations = rlp.at(3)?.as_list()?;
        Ok(XCallEnvelope {
            msg_type,
            message,
            sources,
            destinations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlp_encode_decode_xcall_envelope() {
        let envelope = XCallEnvelope {
            msg_type: 0,
            message: vec![0x01, 0x02, 0x03, 0x04],
            // sources: vec!["source1".to_string(), "source2".to_string()],
            // destinations: vec!["destination1".to_string(), "destination2".to_string()],
            sources: vec!["7WRpFWLhZ9cZDtuyGqVA93YQacydHKXirqY9cYZaDaZd".to_string()],
            destinations: vec!["cx7235a0296f4f0323587c1840181afbee84bbc91a".to_string()],
        };

        let encoded = rlp::encode(&envelope);

        let hex_string = hex::encode(&encoded);
        println!("Hexadecimal representation: {}", hex_string);
        let decoded: XCallEnvelope = rlp::decode(&encoded).unwrap();

        assert_eq!(decoded.msg_type, 0);
        assert_eq!(decoded.message, envelope.message);
        assert_eq!(decoded.sources, envelope.sources);
        assert_eq!(decoded.destinations, envelope.destinations);
    }
}
