use anchor_lang::prelude::*;
use xcall_lib::xcall_connection_type;

use crate::{constants, error::*};

/// The `Config` state of the centralized connection - the inner data of the
/// program-derived address
#[account]
pub struct Config {
    pub admin: Pubkey,
    pub xcall: Pubkey,
    pub validators: Vec<Pubkey>,
    pub threshold: u8,
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
            validators: Vec::new(),
            threshold: 0,
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

    pub fn get_claimable_fees(&self, account: &AccountInfo) -> Result<u64> {
        let rent = Rent::default();
        let rent_exempt_balance = rent.minimum_balance(Config::LEN);

        Ok(account.lamports() - rent_exempt_balance)
    }

    pub fn get_threshold(&self) -> Result<u8> {
        Ok(self.threshold)
    }

    pub fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold
    }

    pub fn add_validator(&mut self, validator: Pubkey) {
        self.validators.push(validator);
    }

    pub fn remove_validator(&mut self, validator: Pubkey) {
        if self.admin == validator {
            return;
        }
        if self.validators.len() < self.threshold as usize {
            return;
        }
        self.validators.retain(|x| *x != validator);
    }

    pub fn get_validators(&self) -> Vec<Pubkey> {
        self.validators.clone()
    }
}

#[account]
pub struct NetworkFee {
    pub message_fee: u64,
    pub response_fee: u64,
    pub bump: u8,
}

impl NetworkFee {
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
pub struct Receipt {}

impl Receipt {
    pub const SEED_PREFIX: &'static str = "receipt";

    pub const LEN: usize = constants::ACCOUNT_DISCRIMINATOR_SIZE;
}

#[account]
pub struct Authority {
    pub bump: u8,
}

impl Authority {
    pub const SEED_PREFIX: &'static str = xcall_connection_type::CONNECTION_AUTHORITY_SEED;
    pub const LEN: usize = 8 + 1;

    pub fn new(bump: u8) -> Self {
        Self { bump }
    }
}
