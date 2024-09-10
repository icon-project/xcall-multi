use soroban_rlp::{decoder, encoder};
use soroban_sdk::{contracttype, Bytes, Env, String, Vec};
use soroban_xcall_lib::{messages::msg_type::MessageType, network_address::NetworkAddress};

use crate::errors::ContractError;

#[contracttype]
#[derive(Clone, Debug)]
pub struct CSMessageRequest {
    from: String,
    to: String,
    sequence_no: u128,
    msg_type: u32,
    data: Bytes,
    protocols: Vec<String>,
}

impl CSMessageRequest {
    pub fn new(
        from: NetworkAddress,
        to: String,
        sequence_no: u128,
        protocols: Vec<String>,
        message_type: MessageType,
        data: Bytes,
    ) -> Self {
        let msg_type: u8 = message_type.into();
        Self {
            from: from.to_string(),
            to,
            sequence_no,
            msg_type: msg_type as u32,
            data,
            protocols,
        }
    }

    pub fn from(&self) -> NetworkAddress {
        NetworkAddress::from_string(self.from.clone())
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
        let msg_type: u8 = self.msg_type as u8;
        msg_type.into()
    }

    pub fn need_response(&self) -> bool {
        let msg_type: MessageType = (self.msg_type as u8).into();
        msg_type == MessageType::CallMessageWithRollback
    }

    pub fn allow_retry(&self) -> bool {
        let msg_type: MessageType = (self.msg_type as u8).into();
        msg_type == MessageType::CallMessagePersisted
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }

    pub fn hash_data(&mut self, e: &Env) {
        let hash = e.crypto().keccak256(self.data());
        self.data = Bytes::from_array(&e, &hash.to_array())
    }

    pub fn set_protocols(&mut self, protocols: Vec<String>) {
        self.protocols = protocols
    }

    pub fn encode(&self, e: &Env) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);

        list.push_back(encoder::encode_string(&e, self.from.clone()));
        list.push_back(encoder::encode_string(&e, self.to.clone()));
        list.push_back(encoder::encode_u128(&e, self.sequence_no));
        list.push_back(encoder::encode_u8(&e, self.msg_type as u8));
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
            from,
            to,
            sequence_no,
            msg_type,
            data,
            protocols,
        })
    }
}
