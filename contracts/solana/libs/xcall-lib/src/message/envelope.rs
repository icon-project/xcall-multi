use super::*;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Envelope {
    pub message: AnyMessage,
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

impl Envelope {
    pub fn new(msg: AnyMessage, sources: Vec<String>, destinations: Vec<String>) -> Self {
        Self {
            message: msg,
            sources,
            destinations,
        }
    }
}

impl Encodable for Envelope {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        stream.begin_list(4);
        stream.append(&Into::<u8>::into(self.message.msg_type().clone()));
        stream.append(&self.message.to_bytes().unwrap());
        stream.begin_list(self.sources.len());
        for source in self.sources.iter() {
            stream.append(source);
        }
        stream.begin_list(self.destinations.len());
        for dest in self.destinations.iter() {
            stream.append(dest);
        }
    }
}

impl Decodable for Envelope {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let msg_int: u8 = rlp.val_at(0)?;
        let msg_type = MessageType::from(msg_int);
        let message_bytes: Vec<u8> = rlp.val_at(1)?;
        let message = decode_message(msg_type, message_bytes)?;

        let sources = rlp.at(2)?;
        let sources: Vec<String> = sources.as_list()?;
        let destinations = rlp.at(3)?;
        let destinations: Vec<String> = destinations.as_list()?;

        Ok(Envelope {
            message,
            sources,
            destinations,
        })
    }
}

pub fn decode_message(msg_type: MessageType, bytes: Vec<u8>) -> Result<AnyMessage, DecoderError> {
    match msg_type {
        MessageType::CallMessage => {
            let msg: CallMessage = rlp::decode(&bytes)?;
            Ok(AnyMessage::CallMessage(msg))
        }
        MessageType::CallMessageWithRollback => {
            let msg: CallMessageWithRollback = rlp::decode(&bytes)?;
            Ok(AnyMessage::CallMessageWithRollback(msg))
        }
        MessageType::CallMessagePersisted => {
            let msg: CallMessagePersisted = rlp::decode(&bytes)?;
            Ok(AnyMessage::CallMessagePersisted(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_message() {
        let msg_bytes = CallMessagePersisted {
            data: vec![1, 2, 3],
        };

        let encoded = msg_bytes.to_bytes();

        // Decode the message
        let decoded_message =
            decode_message(MessageType::CallMessagePersisted, encoded.clone().unwrap()).unwrap();

        assert_eq!(decoded_message.data(), msg_bytes.data);
    }
}
