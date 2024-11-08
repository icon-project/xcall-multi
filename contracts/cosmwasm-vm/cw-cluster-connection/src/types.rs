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
}
impl Encodable for SignableMsg {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        stream.begin_list(3);
        stream.append(&self.src_network);
        stream.append(&self.conn_sn);
        stream.append(&self.data);
    }
}

#[test]
pub fn test_signable_msg_rlp() {
    let signed_msg = SignableMsg {
        src_network: "0x2.icon".to_string(),
        conn_sn: 456456,
        data: "hello".as_bytes().to_vec(),
    };

    let signed_msg = rlp::encode(&signed_msg).to_vec();

    println!("singed message: {:?}", signed_msg);
}
