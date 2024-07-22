use anchor_lang::prelude::*;
use xcall_lib::network_address::NetworkAddress;

use crate::{dapp, error::XcallError, event, id, state::*};

pub fn execute_rollback<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteRollbackCtx<'info>>,
    sn: u128,
) -> Result<()> {
    let rollback = &ctx.accounts.rollback_account.rollback;

    if !rollback.enabled() {
        return Err(XcallError::RollbackNotEnabled.into());
    }

    let protocols = if rollback.protocols().len() > 0 {
        Some(rollback.protocols().to_owned())
    } else {
        None
    };

    let ix_data = dapp::get_handle_call_message_ix_data(
        NetworkAddress::new(&ctx.accounts.config.network_id, &id().to_string()),
        rollback.rollback().to_owned(),
        protocols,
    )?;

    dapp::invoke_handle_call_message_ix(
        rollback.from().to_owned(),
        ix_data,
        &ctx.accounts.config,
        &ctx.accounts.signer,
        &ctx.accounts.system_program,
        &ctx.remaining_accounts,
    )?;

    emit!(event::RollbackExecuted { sn });

    Ok(())
}

#[derive(Accounts)]
#[instruction(sn : u128,)]
pub struct ExecuteRollbackCtx<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
        has_one = admin @ XcallError::InvalidAdminKey
    )]
    pub config: Account<'info, Config>,

    /// CHECK : need to be the owner of the pda
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sn.to_be_bytes()],
        bump = rollback_account.bump,
        close = admin,
    )]
    pub rollback_account: Account<'info, RollbackAccount>,
}
