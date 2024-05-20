use anchor_lang::prelude::{borsh::{BorshDeserialize, BorshSerialize}, *};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RollbackData {
    pub from: Pubkey,
    pub to: String,
    pub protocols: Vec<String>,
    pub rollback: Vec<u8>,
    pub enabled: bool,
}

impl RollbackData {
    pub fn new(
        from: Pubkey,
        to: String,
        protocols: Vec<String>,
        rollback: Vec<u8>,
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
}
