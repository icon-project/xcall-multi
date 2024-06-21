use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
pub struct ConfigCtx<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 1048,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = signer,
        space = 8 + 1024,
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
        init,
        payer = signer,
        space = 8 + DefaultConnection::INIT_SPACE,
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
