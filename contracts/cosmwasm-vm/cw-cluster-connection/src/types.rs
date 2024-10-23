use super::*;

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
