use serde::Serialize;

#[derive(Clone,Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum MessageType {
    BasicMessage = 1,
    MessageWithRollback = 2,
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            Self::BasicMessage => 1,
            Self::MessageWithRollback => 2,
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::BasicMessage,
            2 => MessageType::MessageWithRollback,
            _ => panic!("unsupported message type"),
        }
    }
}
