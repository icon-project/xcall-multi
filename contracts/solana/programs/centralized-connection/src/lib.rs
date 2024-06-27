use std::ops::DerefMut;

use anchor_lang::prelude::*;

pub mod constants;
pub mod contexts;
pub mod error;
pub mod event;
pub mod helper;
pub mod state;

use contexts::*;
use state::*;

declare_id!("7WSWLuAJrg9am6iXTjACUVAgtJTsGctducNw8aYuxdJ6");

#[program]
pub mod centralized_connection {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, xcall: Pubkey, admin: Pubkey) -> Result<()> {
        ctx.accounts
            .config
            .set_inner(Config::new(xcall, admin, ctx.bumps.config));

        ctx.accounts.claim_fee.set_inner(ClaimFee {
            bump: ctx.bumps.claim_fee,
        });

        Ok(())
    }

    pub fn send_message(
        ctx: Context<SendMessage>,
        to: String,
        sn: i64,
        msg: Vec<u8>,
    ) -> Result<()> {
        let next_conn_sn = ctx.accounts.config.get_next_conn_sn()?;

        let mut fee = 0;
        if sn >= 0 {
            fee = ctx.accounts.network_fee.get(sn > 0)?;
        }

        if fee > 0 {
            helper::transfer_lamports(
                &ctx.accounts.signer,
                &ctx.accounts.claim_fee.to_account_info(),
                &ctx.accounts.system_program,
                fee,
            )?
        }

        emit!(event::SendMessage {
            targetNetwork: to,
            connSn: next_conn_sn,
            msg: msg
        });

        Ok(())
    }

    pub fn set_admin(ctx: Context<SetAdmin>, account: Pubkey) -> Result<()> {
        let config = ctx.accounts.config.deref_mut();
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
            .fee
            .set_inner(Fee::new(message_fee, response_fee, ctx.bumps.fee));

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn get_fee(ctx: Context<GetFee>, network_id: String, response: bool) -> Result<u64> {
        ctx.accounts.fee.get(response)
    }

    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        let claim_fees = ctx.accounts.claim_fees.to_account_info();
        let fee = ctx.accounts.claim_fees.get_claimable_fees(&claim_fees)?;

        **claim_fees.try_borrow_mut_lamports()? -= fee;
        **ctx.accounts.admin.try_borrow_mut_lamports()? += fee;

        Ok(())
    }
}
