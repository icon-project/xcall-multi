use soroban_sdk::{contracttype, Bytes, Env};

use crate::errors::ContractError;

#[contracttype]
#[derive(Clone, Copy)]
pub enum CSResponseType {
    CSResponseFailure = 0,
    CSResponseSuccess = 1,
}

impl From<CSResponseType> for u32 {
    fn from(value: CSResponseType) -> Self {
        match value {
            CSResponseType::CSResponseFailure => 0,
            CSResponseType::CSResponseSuccess => 1,
        }
    }
}

impl TryFrom<u32> for CSResponseType {
    type Error = ContractError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CSResponseType::CSResponseFailure),
            1 => Ok(CSResponseType::CSResponseSuccess),
            _ => Err(ContractError::InvalidType),
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
    pub fn new(
        e: &Env,
        sequence_no: u128,
        response_code: CSResponseType,
        message: Option<Bytes>,
    ) -> Self {
        Self {
            sequence_no,
            response_code,
            message: message.unwrap_or(Bytes::new(&e)),
        }
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn response_code(&self) -> &CSResponseType {
        &self.response_code
    }

    // TODO: rlp decode message and return
    pub fn message(&self) -> Option<Bytes> {
        if self.message.is_empty() {
            return None;
        }

        Some(self.message.clone())
    }
}
