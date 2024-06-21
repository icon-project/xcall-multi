use anchor_lang::prelude::*;
use std::ops::DerefMut;

pub mod constants;
pub mod contexts;
pub mod error;
pub mod state;

use contexts::*;
use state::*;

declare_id!("CgXQcZ26YLCoqM1wUK4nCXBwtbNeVZoZgt8ueVJ8Bva1");

#[program]
pub mod centralized_connection {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, xcall: Pubkey, admin: Pubkey) -> Result<()> {
        ctx.accounts
            .config
            .set_inner(Config::new(xcall, admin, ctx.bumps.config));

        Ok(())
    }

    pub fn set_admin(ctx: Context<SetAdmin>, account: Pubkey) -> Result<()> {
        let config = ctx.accounts.config.deref_mut();
        config.ensure_admin(ctx.accounts.signer.key())?;

        config.admin = account;

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_fee(
        ctx: Context<SetFee>,
        network_id: String,
        message_fee: u64,
        response_fee: u64,
    ) -> Result<()> {
        ctx.accounts
            .config
            .ensure_admin(ctx.accounts.signer.key())?;

        ctx.accounts
            .fee
            .set_inner(Fee::new(message_fee, response_fee, ctx.bumps.fee));

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn get_fee(ctx: Context<GetFee>, network_id: String, response: bool) -> Result<u64> {
        Ok(ctx.accounts.fee.get(response))
    }

    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        ctx.accounts
            .config
            .ensure_admin(ctx.accounts.signer.key())?;

        let fee = get_claimable_fees(&ctx.accounts.claim_fees)?;

        **ctx.accounts.claim_fees.try_borrow_mut_lamports()? -= fee;
        **ctx.accounts.signer.try_borrow_mut_lamports()? += fee;

        Ok(())
    }
}
