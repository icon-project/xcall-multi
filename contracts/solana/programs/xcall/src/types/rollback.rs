use anchor_lang::{
    prelude::{borsh, Pubkey},
    AnchorDeserialize, AnchorSerialize,
};
use xcall_lib::network_address::NetworkAddress;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Rollback {
    from: Pubkey,
    to: NetworkAddress,
    protocols: Vec<String>,
    rollback: Vec<u8>,
    enabled: bool,
}

impl Rollback {
    pub fn new(
        from: Pubkey,
        to: NetworkAddress,
        protocols: Vec<String>,
        rollback: Vec<u8>,
        enabled: bool,
    ) -> Self {
        Self {
            from,
            to,
            rollback,
            protocols,
            enabled,
        }
    }

    pub fn from(&self) -> &Pubkey {
        &self.from
    }

    pub fn to(&self) -> &NetworkAddress {
        &self.to
    }

    pub fn rollback(&self) -> &[u8] {
        &self.rollback
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn protocols(&self) -> &Vec<String> {
        &self.protocols
    }

    pub fn enable_rollback(&mut self) {
        self.enabled = true;
    }
}
