use anchor_lang::{prelude::*, solana_program::keccak};

use crate::RollbackDataAccount;

#[derive(Accounts)]
pub struct Rollback<'info> {
    #[account(mut)]
    pub rollback_data: Account<'info, RollbackDataAccount>,

    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn execute_rollback(ctx: Context<Rollback>, sn: u128) -> Result<()> {
    let req = &ctx.accounts.rollback_data;

    // require_eq!(req.to, "".to_owned());

    // check if rollback is enabled
    // require!(req.enabled, ErrorCode::);
    // require!(req.enabled);

    Ok(())
}
pub fn execute_rollback_result() -> Result<()> {
    // emit!(RollbackExecuted { sn: sn });
    Ok(())
}

#[event]
pub struct RollbackExecuted {
    pub sn: u128,
}
