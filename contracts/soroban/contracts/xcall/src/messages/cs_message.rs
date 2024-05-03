use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, vec, Bytes, Env};

use crate::errors::ContractError;
use crate::types::request::CSMessageRequest;
use crate::types::result::CSMessageResult;

#[contracttype]
#[derive(Clone, Copy, Debug)]
pub enum CSMessageType {
    CSMessageRequest = 1,
    CSMessageResult = 0,
}

impl From<CSMessageType> for u32 {
    fn from(value: CSMessageType) -> Self {
        match value {
            CSMessageType::CSMessageRequest => 1,
            CSMessageType::CSMessageResult => 0,
        }
    }
}

impl TryFrom<u32> for CSMessageType {
    type Error = ContractError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(CSMessageType::CSMessageRequest),
            0 => Ok(CSMessageType::CSMessageResult),
            _ => Err(ContractError::InvalidMessageType),
        }
    }
}

#[contracttype]
#[derive(Clone, Debug)]
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

    pub fn encode(env: &Env, msg: &CSMessage) -> Bytes {
        let mut list = vec![&env];

        list.push_back(encoder::encode_u32(&env, msg.message_type.into()));
        list.push_back(encoder::encode(&env, msg.payload.clone()));

        encoder::encode_list(&env, list, false)
    }

    pub fn decode(env: &Env, bytes: Bytes) -> Result<Self, ContractError> {
        let decoded = decoder::decode_list(&env, bytes);
        if decoded.len() != 2 {
            return Err(ContractError::InvalidMessage);
        }

        let message_type: CSMessageType =
            decoder::decode_u32(&env, decoded.get(0).unwrap()).try_into()?;
        let payload = decoded.get(1).unwrap();

        Ok(Self {
            message_type,
            payload,
        })
    }
}
