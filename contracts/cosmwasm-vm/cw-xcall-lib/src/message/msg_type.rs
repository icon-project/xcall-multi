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

#[cfg(test)]
mod tests {
    use crate::message::msg_type::MessageType;

    #[test]
    fn test_message_type_for_u8() {
        assert_eq!(MessageType::CallMessage, 0.into());
        assert_eq!(MessageType::CallMessagePersisted, 2.into());
        assert_eq!(MessageType::CallMessageWithRollback, 1.into())
    }

    #[test]
    fn test_message_type_from_int() {
        assert_eq!(MessageType::from_int(0), MessageType::CallMessage);
        assert_eq!(
            MessageType::from_int(1),
            MessageType::CallMessageWithRollback
        );
        assert_eq!(MessageType::from_int(2), MessageType::CallMessagePersisted)
    }

    #[test]
    #[should_panic(expected = "unsupported message type")]
    fn test_message_type_from_int_fail() {
        MessageType::from_int(4);
    }
}
