use super::*;
use common::rlp::Nullable;
use cosmwasm_std::Addr;
use cw_xcall_lib::{message::msg_type::MessageType, network_address::NetworkAddress};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CSMessageRequest {
    from: NetworkAddress,
    to: Addr,
    sequence_no: u128,
    protocols: Vec<String>,
    msg_type: MessageType,
    data: Nullable<Vec<u8>>,
}

impl CSMessageRequest {
    pub fn new(
        from: NetworkAddress,
        to: Addr,
        sequence_no: u128,
        msg_type: MessageType,
        data: Vec<u8>,
        protocols: Vec<String>,
    ) -> Self {
        let data_bytes = match data.is_empty() {
            true => None,
            false => Some(data),
        };
        Self {
            from,
            to,
            sequence_no,
            msg_type,
            data: Nullable::new(data_bytes),
            protocols,
        }
    }

    pub fn from(&self) -> &NetworkAddress {
        &self.from
    }

    pub fn to(&self) -> &Addr {
        &self.to
    }

    pub fn sequence_no(&self) -> u128 {
        self.sequence_no
    }

    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }

    pub fn need_response(&self) -> bool {
        self.msg_type == MessageType::CallMessageWithRollback
    }

    pub fn allow_retry(&self) -> bool {
        self.msg_type == MessageType::CallMessagePersisted
    }

    pub fn data(&self) -> Result<&[u8], ContractError> {
        Ok(self
            .data
            .get()
            .map_err(|error| ContractError::DecodeFailed {
                error: error.to_string(),
            })?)
    }

    pub fn protocols(&self) -> &Vec<String> {
        &self.protocols
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }
}

impl Encodable for CSMessageRequest {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        stream.begin_list(6);
        stream.append(&self.from.to_string());
        stream.append(&self.to.to_string());
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
        let rlp_protocols = rlp.at(5)?;
        let list: Vec<String> = rlp_protocols.as_list()?;
        let str_from: String = rlp.val_at(0)?;
        let to_str: String = rlp.val_at(1)?;
        let msg_type_int: u8 = rlp.val_at(3)?;
        Ok(Self {
            from: NetworkAddress::from_str(&str_from)
                .map_err(|_e| rlp::DecoderError::RlpInvalidLength)?,
            to: Addr::unchecked(to_str),
            sequence_no: rlp.val_at(2)?,
            msg_type: MessageType::from_int(msg_type_int),
            data: rlp.val_at(4)?,
            protocols: list,
        })
    }
}

impl TryFrom<&Vec<u8>> for CSMessageRequest {
    type Error = ContractError;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value as &[u8]);
        Self::decode(&rlp).map_err(|error| ContractError::DecodeFailed {
            error: error.to_string(),
        })
    }
}

impl TryFrom<&[u8]> for CSMessageRequest {
    type Error = ContractError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let rlp = rlp::Rlp::new(value);
        Self::decode(&rlp).map_err(|error| ContractError::DecodeFailed {
            error: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {

    /*
    CSMessageRequest
     from: 0x1.ETH/0xa
     to: cx0000000000000000000000000000000000000102
     sn: 21
     messageType: 1
     data: 74657374
     protocol: []
     RLP: f83f8b3078312e4554482f307861aa63783030303030303030303030303030303030303030303030303030303030303030303030303031303215018474657374c0

     CSMessageRequest
     from: 0x1.ETH/0xa
     to: cx0000000000000000000000000000000000000102
     sn: 21
     messageType: 1
     data: 74657374
     protocol: [abc, cde, efg]
     RLP: f84b8b3078312e4554482f307861aa63783030303030303030303030303030303030303030303030303030303030303030303030303031303215018474657374cc836162638363646583656667

     CSMessageRequest
     from: 0x1.ETH/0xa
     to: cx0000000000000000000000000000000000000102
     sn: 21
     messageType: 2
     data: 74657374
     protocol: [abc, cde, efg]
     RLP: f84b8b3078312e4554482f307861aa63783030303030303030303030303030303030303030303030303030303030303030303030303031303215028474657374cc836162638363646583656667


     */

    use std::str::FromStr;

    use common::rlp::{self, RlpStream};
    use cosmwasm_std::Addr;
    use cw_xcall_lib::network_address::NetworkAddress;

    use super::CSMessageRequest;
    use cw_xcall_lib::message::msg_type::MessageType;

    #[test]
    fn test_csmessage_request_encoding() {
        let data = hex::decode("74657374").unwrap();
        let msg = CSMessageRequest::new(
            NetworkAddress::from_str("0x1.ETH/0xa").unwrap(),
            Addr::unchecked("cx0000000000000000000000000000000000000102"),
            21,
            MessageType::CallMessage,
            data.clone(),
            vec![],
        );

        let encoded = rlp::encode(&msg);

        assert_eq!("f83f8b3078312e4554482f307861aa63783030303030303030303030303030303030303030303030303030303030303030303030303031303215008474657374c0",hex::encode(encoded));

        let msg = CSMessageRequest::new(
            NetworkAddress::from_str("0x1.ETH/0xa").unwrap(),
            Addr::unchecked("cx0000000000000000000000000000000000000102"),
            21,
            MessageType::CallMessage,
            data.clone(),
            vec!["abc".to_string(), "cde".to_string(), "efg".to_string()],
        );

        let encoded = rlp::encode(&msg);

        assert_eq!("f84b8b3078312e4554482f307861aa63783030303030303030303030303030303030303030303030303030303030303030303030303031303215008474657374cc836162638363646583656667",hex::encode(encoded));

        let msg = CSMessageRequest::new(
            NetworkAddress::from_str("0x1.ETH/0xa").unwrap(),
            Addr::unchecked("cx0000000000000000000000000000000000000102"),
            21,
            MessageType::CallMessageWithRollback,
            data,
            vec!["abc".to_string(), "cde".to_string(), "efg".to_string()],
        );

        let encoded = rlp::encode(&msg);

        assert_eq!("f84b8b3078312e4554482f307861aa63783030303030303030303030303030303030303030303030303030303030303030303030303031303215018474657374cc836162638363646583656667",hex::encode(encoded));
    }

    #[test]
    fn test_network_address() {
        let addr = NetworkAddress::from_str("0x1.ETH/0xa").unwrap();
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&addr.to_string());
        let bytes = hex::encode(&rlp_stream.out());
        println!("{:?}", bytes);
        assert_eq!("8b3078312e4554482f307861", &bytes);
    }

    #[test]
    fn test_sn_encode() {
        let addr = NetworkAddress::from_str("0x1.ETH/0xa").unwrap();
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&21);
        let bytes = hex::encode(&rlp_stream.out());
        println!("{:?}", bytes);
        assert_eq!("15", &bytes);
    }

    #[test]
    fn test_addr() {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&"cx0000000000000000000000000000000000000102".to_string());
        let bytes = hex::encode(&rlp_stream.out());
        println!("{:?}", bytes);
        assert_eq!("aa637830303030303030303030303030303030303030303030303030303030303030303030303030313032",&bytes);
    }

    #[test]
    fn test_bytes() {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&hex::decode("74657374").unwrap());
        let bytes = hex::encode(&rlp_stream.out());
        println!("{:?}", bytes);
        assert_eq!("8474657374", &bytes);
    }

    #[test]
    fn test_list() {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&vec![]);
        let bytes = hex::encode(&rlp_stream.out());
        println!("{:?}", bytes);
        assert_eq!("80", &bytes);
    }
}
