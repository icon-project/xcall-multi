use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

use super::network_address::NetworkAddress;

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    CallMessage = 0,
    CallMessageWithRollback = 1,
    CallMessagePersisted = 2,
}

#[contracttype]
pub struct CallMessage {
    pub data: Bytes,
}

#[contracttype]
pub struct CallMessageWithRollback {
    pub data: Bytes,
    pub rollback: Bytes,
}

#[contracttype]
pub struct CallMessagePersisted {
    pub data: Bytes,
}

#[contracttype]
pub struct Envelope {
    pub message: AnyMessage,
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

#[contracttype]
pub struct InitializeMsg {
    pub network_id: String,
    pub sender: Address,
    pub native_token: Address,
}

#[contracttype]
pub struct Rollback {
    pub from: Address,
    pub to: NetworkAddress,
    pub protocols: Vec<String>,
    pub rollback: Bytes,
    pub enabled: bool,
}

impl Rollback {
    pub fn new(
        from: Address,
        to: NetworkAddress,
        protocols: Vec<String>,
        rollback: Bytes,
        enabled: bool,
    ) -> Self {
        Self {
            from,
            to,
            protocols,
            rollback,
            enabled,
        }
    }

    pub fn from(&self) -> &Address {
        &self.from
    }

    pub fn to(&self) -> &NetworkAddress {
        &self.to
    }

    pub fn protocols(&self) -> &Vec<String> {
        &self.protocols
    }

    pub fn rollback(&self) -> &Bytes {
        &self.rollback
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true
    }
}

pub trait IMessage {
    fn data(&self) -> Bytes;
    fn rollback(&self) -> Option<Bytes>;
}

impl IMessage for CallMessageWithRollback {
    fn data(&self) -> Bytes {
        self.data.clone()
    }

    fn rollback(&self) -> Option<Bytes> {
        Some(self.rollback.clone())
    }
}

impl IMessage for CallMessage {
    fn data(&self) -> Bytes {
        self.data.clone()
    }

    fn rollback(&self) -> Option<Bytes> {
        None
    }
}

impl IMessage for CallMessagePersisted {
    fn data(&self) -> Bytes {
        self.data.clone()
    }

    fn rollback(&self) -> Option<Bytes> {
        None
    }
}

#[contracttype]
pub enum AnyMessage {
    CallMessage(CallMessage),
    CallMessageWithRollback(CallMessageWithRollback),
    CallMessagePersisted(CallMessagePersisted),
}

impl IMessage for AnyMessage {
    fn data(&self) -> Bytes {
        match self {
            AnyMessage::CallMessage(msg) => msg.data(),
            AnyMessage::CallMessagePersisted(msg) => msg.data(),
            AnyMessage::CallMessageWithRollback(msg) => msg.data(),
        }
    }

    fn rollback(&self) -> Option<Bytes> {
        match self {
            AnyMessage::CallMessage(_) => None,
            AnyMessage::CallMessagePersisted(_) => None,
            AnyMessage::CallMessageWithRollback(msg) => msg.rollback(),
        }
    }
}

impl AnyMessage {
    pub fn msg_type(&self) -> MessageType {
        match self {
            AnyMessage::CallMessage(_) => MessageType::CallMessage,
            AnyMessage::CallMessageWithRollback(_) => MessageType::CallMessageWithRollback,
            AnyMessage::CallMessagePersisted(_) => MessageType::CallMessagePersisted,
        }
    }
}
