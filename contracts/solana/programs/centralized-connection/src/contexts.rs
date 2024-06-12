use anchor_lang::prelude::*;

use super::id;
use crate::{constants, state::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Config
    #[account(
        init,
        payer = signer,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
        space = Config::LEN
    )]
    pub config: Account<'info, Config>,

    /// CHECK: PDA account to hold lamports
    #[account(
        init,
        payer = signer,
        seeds = [constants::CLAIM_FEES_SEED_PREFIX.as_bytes()],
        space = constants::ACCOUNT_DISCRIMINATOR_SIZE,
        bump
    )]
    pub claim_fee: AccountInfo<'info>,

    /// Rent payer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// System Program: Required for creating the centralized-connection config
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// Transaction signer
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct SetFee<'info> {
    /// Fee
    #[account(
        init_if_needed,
        payer = signer,
        seeds = [Fee::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump,
        space = Fee::LEN
    )]
    pub fee: Account<'info, Fee>,

    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// Rent payer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// System Program: Required to create program-derived address
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct GetFee<'info> {
    /// Fee
    #[account(
        seeds = [Fee::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump = fee.bump
    )]
    pub fee: Account<'info, Fee>,
}

#[derive(Accounts)]
pub struct ClaimFees<'info> {
    /// Config
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK:
    #[account(
        mut,
        owner = id()
    )]
    pub claim_fees: UncheckedAccount<'info>,

    /// Rent payer
    #[account(mut)]
    pub signer: Signer<'info>,
}
