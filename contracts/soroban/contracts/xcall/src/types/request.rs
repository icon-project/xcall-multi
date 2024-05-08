use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, Bytes, Env, String, Vec};

use super::{message::MessageType, network_address::NetworkAddress};
use crate::errors::ContractError;

#[contracttype]
#[derive(Clone, Debug)]
pub struct CSMessageRequest {
    from: NetworkAddress,
    to: String,
    sequence_no: u128,
    protocols: Vec<String>,
    msg_type: MessageType,
    data: Bytes,
}

impl CSMessageRequest {
    pub fn new(
        from: NetworkAddress,
        to: String,
        sequence_no: u128,
        protocols: Vec<String>,
        msg_type: MessageType,
        data: Bytes,
    ) -> Self {
        Self {
            from,
            to,
            sequence_no,
            protocols,
            msg_type,
            data,
        }
    }

    pub fn from(&self) -> &NetworkAddress {
        &self.from
    }

    pub fn to(&self) -> &String {
        &self.to
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn protocols(&self) -> &Vec<String> {
        &self.protocols
    }

    pub fn msg_type(&self) -> MessageType {
        self.msg_type
    }

    pub fn need_response(&self) -> bool {
        self.msg_type == MessageType::CallMessageWithRollback
    }

    pub fn allow_retry(&self) -> bool {
        self.msg_type == MessageType::CallMessagePersisted
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }

    pub fn hash_data(&mut self, e: &Env) {
        let hash = e.crypto().keccak256(self.data());
        self.data = Bytes::from_array(&e, &hash.to_array())
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);

        list.push_back(encoder::encode_string(&e, self.from.as_string().clone()));
        list.push_back(encoder::encode_string(&e, self.to.clone()));
        list.push_back(encoder::encode_u128(&e, self.sequence_no));
        list.push_back(encoder::encode_u8(&e, self.msg_type.into()));
        list.push_back(encoder::encode(&e, self.data.clone()));
        list.push_back(encoder::encode_strings(&e, self.protocols.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> Result<CSMessageRequest, ContractError> {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 6 {
            return Err(ContractError::InvalidRlpLength);
        }

        let from = decoder::decode_string(e, decoded.get(0).unwrap());
        let to = decoder::decode_string(&e, decoded.get(1).unwrap());
        let sequence_no = decoder::decode_u128(&e, decoded.get(2).unwrap());
        let msg_type = decoder::decode_u8(&e, decoded.get(3).unwrap()).into();
        let data = decoded.get(4).unwrap();
        let protocols = decoder::decode_strings(&e, decoded.get(5).unwrap());

        Ok(Self {
            from: NetworkAddress::to_network_address(from),
            to,
            sequence_no,
            msg_type,
            data,
            protocols,
        })
    }
}
