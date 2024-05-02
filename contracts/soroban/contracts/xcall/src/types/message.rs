use soroban_sdk::{contracttype, Bytes};

use super::{request::CSMessageRequest, result::CSMessageResult};

#[contracttype]
#[derive(Clone, Copy)]
pub enum CSMessageType {
    CSMessageRequest = 1,
    CSMessageResult = 0,
}

#[contracttype]
#[derive(Clone)]
pub struct CSMessage {
    message_type: CSMessageType,
    payload: Bytes,
}

impl From<CSMessageRequest> for CSMessage {
    // TODO: rlp encode value as payload
    fn from(value: CSMessageRequest) -> Self {
        Self {
            message_type: CSMessageType::CSMessageRequest,
            payload: value.data().clone(),
        }
    }
}

impl From<CSMessageResult> for CSMessage {
    // TODO: rlp encode value as payload
    fn from(value: CSMessageResult) -> Self {
        Self {
            message_type: CSMessageType::CSMessageResult,
            payload: value.message().clone().unwrap(),
        }
    }
}

impl CSMessage {
    pub fn message_type(&self) -> &CSMessageType {
        &self.message_type
    }

    pub fn payload(&self) -> &Bytes {
        &self.payload
    }
}
