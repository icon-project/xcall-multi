use anchor_lang::prelude::*;

use crate::state::*;

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
pub struct UpdateConfigCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct DefaultConnectionCtx<'info> {
    #[account(
        init_if_needed,
        payer = signer,
        space = DefaultConnection::SIZE,
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}