use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, vec, Bytes, Env, Vec};

#[contracttype]
#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    FILL = 1,
    CANCEL = 2,
}

impl From<u32> for MessageType {
    fn from(value: u32) -> Self {
        match value {
            1 => MessageType::FILL,
            2 => MessageType::CANCEL,
            _ => panic!("Invalid message type"),
        }
    }
}

impl From<MessageType> for u32 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::FILL => 1,
            MessageType::CANCEL => 2,
        }
    }
}

#[contracttype]
#[derive(Debug, Clone)]
pub struct OrderMessage {
    /// Type of message (Fill or Cancel)
    message_type: MessageType,
    /// Encoded message data
    message: Bytes,
}

impl OrderMessage {
    pub fn new(message_type: MessageType, message: Bytes) -> Self {
        Self {
            message_type,
            message,
        }
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    pub fn message(&self) -> Bytes {
        self.message.clone()
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut list: Vec<Bytes> = vec![&e];

        list.push_back(encoder::encode_u32(&e, self.message_type.into()));
        list.push_back(encoder::encode(&e, self.message()));

        encoder::encode_list(&e, list, false)
    }

    pub fn decode(e: &Env, list: Bytes) -> Self {
        let decoded = decoder::decode_list(&e, list);
        if decoded.len() != 2 {
            panic!("Invalid rlp bytes lenght")
        }

        let msg_type = decoder::decode_u32(&e, decoded.get(0).unwrap());
        let message = decoded.get(1).unwrap();

        Self {
            message_type: msg_type.into(),
            message,
        }
    }
}
