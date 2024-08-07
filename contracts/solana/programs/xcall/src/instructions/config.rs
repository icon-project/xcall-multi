use anchor_lang::prelude::*;

use crate::{error::XcallError, state::*};

pub fn initialize(ctx: Context<ConfigCtx>, network_id: String) -> Result<()> {
    ctx.accounts
        .config
        .new(ctx.accounts.signer.key(), network_id, ctx.bumps.config);

    Ok(())
}

pub fn set_admin(ctx: Context<SetAdminCtx>, account: Pubkey) -> Result<()> {
    ctx.accounts.config.set_admin(account);

    Ok(())
}

#[derive(Accounts)]
pub struct ConfigCtx<'info> {
    #[account(
        init,
        payer = signer,
        space = Config::SIZE,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub signer: Signer<'info>,

    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetConfigCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetAdminCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ XcallError::OnlyAdmin
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,
}
