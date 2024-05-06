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

impl From<u32> for CSMessageType {
    fn from(value: u32) -> Self {
        match value {
            1 => CSMessageType::CSMessageRequest,
            0 => CSMessageType::CSMessageResult,
            _ => panic!("Invalid message type"),
        }
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CSMessage {
    message_type: CSMessageType,
    payload: Bytes,
}

impl CSMessage {
    pub fn message_type(&self) -> &CSMessageType {
        &self.message_type
    }

    pub fn payload(&self) -> &Bytes {
        &self.payload
    }

    pub fn from_request(env: &Env, request: &CSMessageRequest) -> Self {
        Self {
            message_type: CSMessageType::CSMessageRequest,
            payload: request.encode(&env),
        }
    }

    pub fn from_result(env: &Env, result: &CSMessageResult) -> Self {
        Self {
            message_type: CSMessageType::CSMessageResult,
            payload: result.encode(&env),
        }
    }

    pub fn encode(&self, env: &Env) -> Bytes {
        let mut list = vec![&env];

        list.push_back(encoder::encode_u32(&env, self.message_type.into()));
        list.push_back(encoder::encode(&env, self.payload.clone()));

        encoder::encode_list(&env, list, false)
    }

    pub fn decode(env: &Env, bytes: Bytes) -> Result<Self, ContractError> {
        let decoded = decoder::decode_list(&env, bytes);
        if decoded.len() != 2 {
            return Err(ContractError::InvalidRlpLength);
        }

        let payload = decoded.get(1).unwrap();
        let msg_type = decoded.get(0).unwrap();
        let message_type = if msg_type.len() > 0 {
            decoder::decode_u32(&env, decoded.get(0).unwrap()).into()
        } else {
            CSMessageType::CSMessageResult
        };

        Ok(Self {
            message_type,
            payload,
        })
    }
}
