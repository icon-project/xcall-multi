use super::*;
use common::rlp::{self, Encodable};

#[cw_serde]
pub struct InstantiateMsg {
    pub relayer: String,
    pub xcall_address: String,
    pub denom: String,
}

#[cw_serde]
pub enum StorageKey {
    XCall,
    Admin,
    Relayer,
    Validators,
    SignatureThreshold,

    MessageFee,
    ResponseFee,

    ConnSn,
    Receipts,

    Denom,
}

impl StorageKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageKey::XCall => "xcall",
            StorageKey::Admin => "admin",
            StorageKey::Relayer => "relayer",
            StorageKey::Validators => "validators",
            StorageKey::SignatureThreshold => "signature_threshold",

            StorageKey::MessageFee => "message_fee",
            StorageKey::ResponseFee => "response_fee",

            StorageKey::ConnSn => "conn_sn",
            StorageKey::Receipts => "receipts",

            StorageKey::Denom => "denom",
        }
    }
}

pub struct SignableMsg {
    pub src_network: String,
    pub conn_sn: u128,
    pub data: Vec<u8>,
    pub dst_network: String,
}
impl SignableMsg {
    pub fn encode_utf8_bytes(&self) -> Vec<u8> {
        let mut encoded_bytes = Vec::new();

        encoded_bytes.extend(self.src_network.as_bytes());

        encoded_bytes.extend(self.conn_sn.to_string().as_bytes());

        encoded_bytes.extend(self.data.to_vec());

        encoded_bytes.extend(self.dst_network.as_bytes());

        encoded_bytes
    }
}

#[test]
pub fn test_signable_msg_utf8_bytes() {
    let signed_msg = SignableMsg {
        src_network: "0x2.icon".to_string(),
        conn_sn: 128,
        data: "hello".as_bytes().to_vec(),
        dst_network: "archway".to_string(),
    };

    let expected_encoded_hex_str = "3078322e69636f6e31323868656c6c6f61726368776179".to_string();
    let expected_encoded_bytes = hex::decode(expected_encoded_hex_str).unwrap();

    assert_eq!(
        expected_encoded_bytes,
        signed_msg.encode_utf8_bytes(),
        "test failed"
    );
}
