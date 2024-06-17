use super::*;

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct CallMessage {
    pub data: Vec<u8>,
}

impl Encodable for CallMessage {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(1).append(&self.data);
    }
}

impl Decodable for CallMessage {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Self {
            data: rlp.val_at(0)?,
        })
    }
}

impl IMessage for CallMessage {
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
    use super::*;
    use rlp::Rlp;

    #[test]
    fn test_encoding_decoding_message() {
        let original_message = CallMessage {
            data: vec![1, 2, 3, 4],
        };
        let encoded = original_message.to_bytes().unwrap();

        let decoded_rlp = Rlp::new(&encoded);
        let decoded_message = CallMessage::decode(&decoded_rlp).unwrap();

        assert_eq!(decoded_message.data, original_message.data);
    }
}
