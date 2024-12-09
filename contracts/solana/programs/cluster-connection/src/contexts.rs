use anchor_lang::prelude::*;

use crate::{error::ConnectionError, state::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Rent payer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// System Program: Required for creating the centralized-connection config
    pub system_program: Program<'info, System>,

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
        space = Authority::LEN,
        seeds = [Authority::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub authority: Account<'info, Authority>,
}

#[derive(Accounts)]
#[instruction(to: String)]
pub struct SendMessage<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        owner = config.xcall @ ConnectionError::OnlyXcall
    )]
    pub xcall: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [NetworkFee::SEED_PREFIX.as_bytes(), to.as_bytes()],
        bump = network_fee.bump
    )]
    pub network_fee: Account<'info, NetworkFee>,
}

#[derive(Accounts)]
pub struct RevertMessage<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ ConnectionError::OnlyAdmin,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [Authority::SEED_PREFIX.as_bytes()],
        bump = authority.bump
    )]
    pub authority: Account<'info, Authority>,
}

#[derive(Accounts)]
pub struct SetConfigItem<'info> {
    /// Transaction signer
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
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct SetFee<'info> {
    /// Rent payer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// System Program: Required to create program-derived address
    pub system_program: Program<'info, System>,

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
    /// Rent payer
    #[account(mut)]
    pub relayer: Signer<'info>,

    /// Config
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = relayer @ ConnectionError::OnlyRelayer,
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct GetConfigItem<'info> {
    /// Config
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(src_network: String, conn_sn: u128)]
pub struct ReceiveMessageWithSignatures<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

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
        seeds = [Receipt::SEED_PREFIX.as_bytes(), src_network.as_bytes(),  &conn_sn.to_be_bytes()],
        space = Receipt::LEN,
        bump
    )]
    pub receipt: Account<'info, Receipt>,

    #[account(
        seeds = [Authority::SEED_PREFIX.as_bytes()],
        bump = authority.bump
    )]
    pub authority: Account<'info, Authority>,
}