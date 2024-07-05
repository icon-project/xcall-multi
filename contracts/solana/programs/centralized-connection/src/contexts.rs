use anchor_lang::prelude::*;

use crate::{constants, error::ConnectionError, state::*};

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

    #[account(
        init,
        payer = signer,
        seeds = [ClaimFee::SEED_PREFIX.as_bytes()],
        space = constants::ACCOUNT_DISCRIMINATOR_SIZE + 1,
        bump
    )]
    pub claim_fee: Account<'info, ClaimFee>,

    /// Rent payer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// System Program: Required for creating the centralized-connection config
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(to: String)]
pub struct SendMessage<'info> {
    #[account(
        owner = config.xcall @ ConnectionError::OnlyXcall
    )]
    pub xcall: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [NetworkFee::SEED_PREFIX.as_bytes(), to.as_bytes()],
        bump = network_fee.bump
    )]
    pub network_fee: Account<'info, NetworkFee>,

    #[account(
        mut,
        seeds = [ClaimFee::SEED_PREFIX.as_bytes()],
        bump = claim_fee.bump
    )]
    pub claim_fee: Account<'info, ClaimFee>,
}

#[derive(Accounts)]
#[instruction(src_network: String, conn_sn: u128)]
pub struct RecvMessage<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ ConnectionError::OnlyAdmin,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        seeds = [Receipt::SEED_PREFIX.as_bytes(), &conn_sn.to_be_bytes()],
        space = Receipt::LEN,
        bump
    )]
    pub receipt: Account<'info, Receipt>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(src_network: String, conn_sn: u128)]
pub struct RevertMessage<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ ConnectionError::OnlyAdmin,
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ ConnectionError::OnlyAdmin,
    )]
    pub config: Account<'info, Config>,

    /// Transaction signer
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct SetFee<'info> {
    /// Fee
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [NetworkFee::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump,
        space = NetworkFee::LEN
    )]
    pub network_fee: Account<'info, NetworkFee>,

    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ ConnectionError::OnlyAdmin,
    )]
    pub config: Account<'info, Config>,

    /// Rent payer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// System Program: Required to create program-derived address
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct GetFee<'info> {
    /// Fee
    #[account(
        seeds = [NetworkFee::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump = network_fee.bump
    )]
    pub network_fee: Account<'info, NetworkFee>,
}

#[derive(Accounts)]
pub struct ClaimFees<'info> {
    /// Config
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ ConnectionError::OnlyAdmin,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [ClaimFee::SEED_PREFIX.as_bytes()],
        bump = claim_fee.bump
    )]
    pub claim_fee: Account<'info, ClaimFee>,

    /// Rent payer
    #[account(mut)]
    pub admin: Signer<'info>,
}
