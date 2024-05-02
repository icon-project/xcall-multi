use super::{message_types::MessageType, network_address::NetworkAddress};

use soroban_sdk::{contracttype, Bytes, String, Vec};

#[contracttype]
#[derive(Clone)]
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
}
