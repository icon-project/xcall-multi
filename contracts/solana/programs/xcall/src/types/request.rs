use super::*;
use std::str::FromStr;

use crate::error::*;
use xcall_lib::{message::msg_type::MessageType, network_address::NetworkAddress};

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct CSMessageRequest {
    from: NetworkAddress,
    to: String,
    sequence_no: u128,
    msg_type: MessageType,
    data: Vec<u8>, // TODO: cosmos this is nullable??
    protocols: Vec<String>,
}

impl CSMessageRequest {
    pub fn new(
        from: NetworkAddress,
        to: String,
        sequence_no: u128,
        msg_type: MessageType,
        data: Vec<u8>,
        protocols: Vec<String>,
    ) -> Self {
        Self {
            from,
            to,
            sequence_no,
            msg_type,
            data,
            protocols,
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

    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn hash_data(&mut self) {
        let hash = solana_program::hash::hash(&self.data());
        self.data = hash.to_bytes().to_vec();
    }

    pub fn set_protocols(&mut self, protocols: Vec<String>) {
        self.protocols = protocols
    }

    pub fn need_response(&self) -> bool {
        self.msg_type == MessageType::CallMessageWithRollback
    }

    pub fn allow_retry(&self) -> bool {
        self.msg_type == MessageType::CallMessagePersisted
    }

    pub fn protocols(&self) -> Vec<String> {
        self.protocols.clone()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }
}

impl Encodable for CSMessageRequest {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        stream.begin_list(6);

        stream.append(&self.from.to_string());
        stream.append(&self.to);
        stream.append(&self.sequence_no);
        stream.append(&self.msg_type.as_int());
        stream.append(&self.data);
        stream.begin_list(self.protocols.len());
        for protocol in self.protocols.iter() {
            stream.append(protocol);
        }
    }
}

impl Decodable for CSMessageRequest {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 6 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        let rlp_protocols = rlp.at(5)?;
        let list: Vec<String> = rlp_protocols.as_list()?;
        let str_from: String = rlp.val_at(0)?;
        let int_msg_type: u8 = rlp.val_at(3)?;

        Ok(Self {
            from: NetworkAddress::from_str(&str_from)
                .map_err(|_e| rlp::DecoderError::RlpInvalidLength)?,
            to: rlp.val_at(1)?,
            sequence_no: rlp.val_at(2)?,
            msg_type: MessageType::from_int(int_msg_type),
            data: rlp.val_at(4)?,
            protocols: list,
        })
    }
}

impl TryFrom<&Vec<u8>> for CSMessageRequest {
    type Error = XcallError;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value as &[u8]);
        Self::decode(&rlp).map_err(|_error| XcallError::DecodeFailed)
    }
}

impl TryFrom<&[u8]> for CSMessageRequest {
    type Error = XcallError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value);
        Self::decode(&rlp).map_err(|_error| XcallError::DecodeFailed)
    }
}
