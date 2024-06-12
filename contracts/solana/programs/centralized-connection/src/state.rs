use anchor_lang::prelude::*;

use crate::constants;
use crate::error::ConnectionError;

/// The `Config` state of the centralized connection - the inner data of the
/// program-derived address
#[account]
pub struct Config {
    pub admin: Pubkey,
    pub xcall: Pubkey,
    pub sn: u128,
    pub bump: u8,
}

impl Config {
    /// The Config seed phrase to derive it's program-derived address
    pub const SEED_PREFIX: &'static str = "config";

    /// Account discriminator + Xcall public key + Admin public key + connection
    /// sequence + bump
    pub const LEN: usize = constants::ACCOUNT_DISCRIMINATOR_SIZE + 32 + 32 + 16 + 1 + 1;

    /// Creates a new centralized connection `Config` state
    pub fn new(xcall: Pubkey, admin: Pubkey, bump: u8) -> Self {
        Self {
            xcall,
            admin,
            sn: 0,
            bump,
        }
    }

    /// It throws error if `signer` is not an admin account
    pub fn ensure_admin(&self, signer: Pubkey) -> Result<()> {
        if self.admin != signer {
            return Err(ConnectionError::OnlAdmin.into());
        }
        Ok(())
    }
}

#[account]
pub struct Fee {
    pub message_fee: u64,
    pub response_fee: u64,
    pub bump: u8,
}

impl Fee {
    /// The Fee seed phrase to derive it's program-derived address
    pub const SEED_PREFIX: &'static str = "fee";

    /// Account discriminator + Message fee + Response fee + bump
    pub const LEN: usize = constants::ACCOUNT_DISCRIMINATOR_SIZE + 8 + 8 + 1;

    /// Creates a new `Fee` state for a network_id
    pub fn new(message_fee: u64, response_fee: u64, bump: u8) -> Self {
        Self {
            message_fee,
            response_fee,
            bump,
        }
    }

    pub fn get(&self, response: bool) -> u64 {
        let mut fee = self.message_fee;
        if response {
            fee += self.response_fee
        }

        fee
    }
}

pub fn get_claimable_fees(fee_account: &UncheckedAccount) -> Result<u64> {
    let rent = Rent::default();
    let rent_exempt_balance = rent.minimum_balance(constants::ACCOUNT_DISCRIMINATOR_SIZE);

    Ok(fee_account.lamports() - rent_exempt_balance)
}
