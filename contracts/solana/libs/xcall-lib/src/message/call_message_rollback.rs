use super::*;

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
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
    use super::*;
    use rlp::Encodable;
    use rlp::Rlp;

    #[test]
    fn test_encoding_decoding_message() {
        let original_message = CallMessageWithRollback {
            data: vec![0, 11, 255],
            rollback: vec![1, 2, 3],
        };

        let mut stream = RlpStream::new();
        original_message.rlp_append(&mut stream);
        let encoded = stream.out();

        let decoded_rlp = Rlp::new(&encoded);
        let decoded_message = CallMessageWithRollback::decode(&decoded_rlp).unwrap();

        assert_eq!(decoded_message.data, original_message.data);
        assert_eq!(decoded_message.rollback, original_message.rollback);
    }
}
