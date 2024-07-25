use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    CallMessage = 0,
    CallMessageWithRollback = 1,
    CallMessagePersisted = 2,
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::CallMessage => 0,
            MessageType::CallMessageWithRollback => 1,
            MessageType::CallMessagePersisted => 2,
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::CallMessage,
            1 => MessageType::CallMessageWithRollback,
            2 => MessageType::CallMessagePersisted,
            _ => panic!("invalid message type"),
        }
    }
}
