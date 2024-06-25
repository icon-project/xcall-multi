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
            return Err(ConnectionError::OnlyAdmin.into());
        }
        Ok(())
    }

    /// It throws error if `address` is not an xcall account
    pub fn ensure_xcall(&self, address: Pubkey) -> Result<()> {
        if self.xcall != address {
            return Err(ConnectionError::OnlyXcall.into());
        }
        Ok(())
    }

    pub fn get_next_conn_sn(&mut self) -> Result<u128> {
        self.sn += 1;
        Ok(self.sn)
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

    pub fn get(&self, response: bool) -> Result<u64> {
        let mut fee = self.message_fee;
        if response {
            fee += self.response_fee
        }

        Ok(fee)
    }
}

#[account]
pub struct ClaimFee {
    pub bump: u8,
}

impl ClaimFee {
    pub const LEN: usize = constants::ACCOUNT_DISCRIMINATOR_SIZE + 1;

    pub fn get_claimable_fees(&self, fee_account: &AccountInfo) -> Result<u64> {
        let rent = Rent::default();
        let rent_exempt_balance = rent.minimum_balance(ClaimFee::LEN);

        Ok(fee_account.lamports() - rent_exempt_balance)
    }
}
