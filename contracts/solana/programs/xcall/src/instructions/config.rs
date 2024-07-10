use anchor_lang::prelude::*;

use crate::{error::XcallError, state::*};

pub fn initialize(ctx: Context<ConfigCtx>, network_id: String) -> Result<()> {
    ctx.accounts
        .config
        .set(ctx.accounts.signer.key(), network_id, ctx.bumps.config);

    ctx.accounts.reply.set();

    Ok(())
}

pub fn set_admin(ctx: Context<SetAdminCtx>, account: Pubkey) -> Result<()> {
    ctx.accounts.config.set_admin(account);

    Ok(())
}

pub fn set_default_connection(
    ctx: Context<DefaultConnectionCtx>,
    connection: Pubkey,
) -> Result<()> {
    ctx.accounts
        .default_connection
        .set(connection, ctx.bumps.default_connection);

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

    #[account(
        init,
        payer = signer,
        space = Reply::SIZE,
        seeds = [Reply::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub reply: Account<'info, Reply>,

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

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct DefaultConnectionCtx<'info> {
    #[account(
        init_if_needed,
        payer = admin,
        space = DefaultConnection::SIZE,
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ XcallError::OnlyAdmin
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
