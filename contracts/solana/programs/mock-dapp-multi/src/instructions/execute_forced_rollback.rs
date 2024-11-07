use anchor_lang::prelude::*;
use xcall_lib::xcall_dapp_type;

use crate::{state::*, xcall};

pub fn execute_forced_rollback<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteForcedRollbackCtx<'info>>,
    req_id: u128,
    from_nid: String,
    conn_sn: u128,
    connection: Pubkey,
) -> Result<()> {
    let ix_data = xcall::get_handle_forced_rollback_ix_data(req_id, from_nid, conn_sn, connection)?;

    xcall::call_xcall_handle_forced_rollback(
        &ix_data,
        &ctx.accounts.config,
        &ctx.accounts.sender,
        &ctx.accounts.authority,
        &ctx.accounts.system_program,
        &ctx.remaining_accounts,
    )
}

#[derive(Accounts)]
pub struct ExecuteForcedRollbackCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [xcall_dapp_type::DAPP_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}
