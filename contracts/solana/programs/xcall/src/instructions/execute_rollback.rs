use anchor_lang::prelude::*;

use crate::{error::XcallError, event, state::*};

pub fn execute_rollback<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteRollbackCtx<'info>>,
    _sn: u128,
) -> Result<()> {
    let req = ctx
        .accounts
        .rollback
        .as_mut()
        .ok_or(XcallError::InvalidSn)?;

    if !req.rollback.enabled() {
        return Err(XcallError::RollbackNotEnabled.into());
    }

    let to = &req.rollback.to();
    let from = &req.rollback.from().to_string();
    let data = &req.rollback.rollback().to_vec();
    let protocols = req.rollback.protocols().to_vec();

    // TODO: need to call on dapp here
    // handle_call_message(ctx,
    //     req,
    //     data,
    //     false,
    //     )?

    emit!(event::RollbackExecuted { sn: _sn });

    Ok(())
}

#[derive(Accounts)]
#[instruction(_sn : u128,)]
pub struct ExecuteRollbackCtx<'info> {
    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &_sn.to_string().as_bytes()],
        bump = rollback.bump,
        close = owner
    )]
    pub rollback: Option<Account<'info, RollbackAccount>>,

    #[account(
        mut,
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub default_connection: Option<Account<'info, DefaultConnection>>,

    #[account(mut)]
    /// CHECK : need to be the owner of the pda
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
