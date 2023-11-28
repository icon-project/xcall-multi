use common::rlp::{self, Decodable, DecoderError, Encodable};

use super::{
    call_message::CallMessage, call_message_persisted::CallMessagePersisted,
    call_message_rollback::CallMessageWithRollback, msg_trait::IMessage, msg_type::MessageType,
    AnyMessage,
};
#[derive(Clone, Debug, PartialEq)]
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
    fn rlp_append(&self, stream: &mut common::rlp::RlpStream) {
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
        MessageType::BasicMessage => {
            let msg: CallMessage = rlp::decode(&bytes)?;
            Ok(AnyMessage::CallMessage(msg))
        }
        MessageType::MessageWithRollback => {
            let msg: CallMessageWithRollback = rlp::decode(&bytes)?;
            Ok(AnyMessage::CallMessageWithRollback(msg))
        }
        MessageType::PersistedMessge => {
            let msg: CallMessagePersisted = rlp::decode(&bytes)?;
            Ok(AnyMessage::CallMessagePersisted(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_decoding_envelope_call_message() {
        // Create a sample Envelope
        let message = AnyMessage::CallMessage(CallMessage {
            msg_type: MessageType::BasicMessage,
            data: vec![1, 2, 3],
        });
        let sources = vec!["source1".to_string(), "source2".to_string()];
        let destinations = vec!["dest1".to_string(), "dest2".to_string()];
        let envelope = Envelope::new(message, sources, destinations);
        let encoded_data = rlp::encode(&envelope).to_vec();

        assert_eq!(
            "e60186c50183010203d087736f757263653187736f7572636532cc856465737431856465737432",
            hex::encode(&encoded_data)
        );
        let decoded: Envelope = rlp::decode(&encoded_data).unwrap();

        assert_eq!(envelope, decoded);
    }

    #[test]
    fn test_encoding_decoding_envelope_call_message_rollback() {
        // Create a sample Envelope
        let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
            msg_type: MessageType::MessageWithRollback,
            data: vec![1, 2, 3],
            rollback: vec![1, 2, 3],
        });
        let sources = vec!["source1".to_string(), "source2".to_string()];
        let destinations = vec!["dest1".to_string(), "dest2".to_string()];
        let envelope = Envelope::new(message, sources, destinations);
        let encoded_data = rlp::encode(&envelope).to_vec();

        assert_eq!("ea028ac5028301020383010203d087736f757263653187736f7572636532cc856465737431856465737432",hex::encode(&encoded_data));
        let decoded: Envelope = rlp::decode(&encoded_data).unwrap();

        assert_eq!(envelope, decoded);
    }
}
