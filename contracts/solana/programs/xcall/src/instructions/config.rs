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
    /// The configuration account, which stores important settings for the program.
    /// This account is initialized only once during the lifetime of program and it will
    /// throw error if tries to initialize twice
    #[account(
        init,
        payer = signer,
        space = Config::SIZE,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetConfigCtx<'info> {
    /// The configuration account, which stores important settings for the program.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetAdminCtx<'info> {
    /// The configuration account, which stores important settings for the program.
    /// This account is mutable because the admin of the program will be updated.
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The account that signs and pays for the transaction. This account is checked
    /// against the `config.admin` to ensure it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::OnlyAdmin
    )]
    pub admin: Signer<'info>,
}
