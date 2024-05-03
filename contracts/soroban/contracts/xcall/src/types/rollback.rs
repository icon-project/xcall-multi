use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

use super::network_address::NetworkAddress;

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
