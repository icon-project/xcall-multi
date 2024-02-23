use serde::Serialize;

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum MessageType {
    CallMessage = 0,
    CallMessageWithRollback = 1,
    CallMessagePersisted = 2,
}

impl From<MessageType> for u8 {
    fn from(val: MessageType) -> Self {
        match val {
            MessageType::CallMessage => 1,
            MessageType::CallMessageWithRollback => 2,
            MessageType::CallMessagePersisted => 3,
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::CallMessage,
            1 => MessageType::CallMessageWithRollback,
            2 => MessageType::CallMessagePersisted,
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
