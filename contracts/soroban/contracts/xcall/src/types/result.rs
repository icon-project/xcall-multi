use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, Bytes, Env, Vec};

use super::request::CSMessageRequest;
use crate::errors::ContractError;

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CSResponseType {
    CSResponseFailure = 0,
    CSResponseSuccess = 1,
}

impl From<CSResponseType> for u8 {
    fn from(value: CSResponseType) -> Self {
        match value {
            CSResponseType::CSResponseFailure => 0,
            CSResponseType::CSResponseSuccess => 1,
        }
    }
}

impl From<u8> for CSResponseType {
    fn from(value: u8) -> Self {
        match value {
            0 => CSResponseType::CSResponseFailure,
            1 => CSResponseType::CSResponseSuccess,
            _ => panic!("Invalid response type"),
        }
    }
}

#[contracttype]
#[derive(Clone)]
pub struct CSMessageResult {
    sequence_no: u128,
    response_code: CSResponseType,
    message: Bytes,
}

impl CSMessageResult {
    pub fn new(sequence_no: u128, response_code: CSResponseType, message: Bytes) -> Self {
        Self {
            sequence_no,
            response_code,
            message,
        }
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn response_code(&self) -> &CSResponseType {
        &self.response_code
    }

    pub fn message(&self, e: &Env) -> Option<CSMessageRequest> {
        if self.message.is_empty() {
            return None;
        }

        CSMessageRequest::decode(&e, self.message.clone()).ok()
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);

        list.push_back(encoder::encode_u128(&e, self.sequence_no.clone()));
        list.push_back(encoder::encode_u8(&e, self.response_code.into()));
        list.push_back(encoder::encode(&e, self.message.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> Result<Self, ContractError> {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 3 {
            return Err(ContractError::InvalidRlpLength);
        }

        let sequence_no = decoder::decode_u128(&e, decoded.get(0).unwrap());
        let response_code = decoder::decode_u8(&e, decoded.get(1).unwrap()).into();
        let message = decoded.get(2).unwrap();

        Ok(Self {
            sequence_no,
            message,
            response_code,
        })
    }
}
