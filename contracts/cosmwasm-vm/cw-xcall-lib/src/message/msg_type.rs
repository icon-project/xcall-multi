use serde::Serialize;

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum MessageType {
    BasicMessage = 1,
    MessageWithRollback = 2,
    PersistedMessge = 3,
}

impl From<MessageType> for u8 {
    fn from(val: MessageType) -> Self {
        match val {
            MessageType::BasicMessage => 1,
            MessageType::MessageWithRollback => 2,
            MessageType::PersistedMessge => 3,
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::BasicMessage,
            2 => MessageType::MessageWithRollback,
            3 => MessageType::PersistedMessge,
            _ => panic!("unsupported message type"),
        }
    }
}

impl MessageType {
    pub fn as_int(&self) -> u8 {
        self.clone().into()
    }
    pub fn from_int(val: u8) -> Self {
        MessageType::from(val)
    }
}
