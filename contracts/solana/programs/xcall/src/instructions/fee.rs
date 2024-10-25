use anchor_lang::prelude::*;
use xcall_lib::xcall_connection_type::{self, GET_FEE_IX};

use crate::{connection, error::*, helper, state::*};

pub fn set_protocol_fee(ctx: Context<SetFeeCtx>, fee: u64) -> Result<()> {
    ctx.accounts.config.set_protocol_fee(fee);

    Ok(())
}

pub fn set_protocol_fee_handler(ctx: Context<SetFeeHandlerCtx>, fee_handler: Pubkey) -> Result<()> {
    ctx.accounts.config.set_fee_handler(fee_handler);

    Ok(())
}

/// Calculates and retrieves the total fee for a cross-chain message, including the protocol fee
/// and connection-specific fees.
///
/// This function computes the total fee required to send a cross-chain message by adding the
/// protocol fee stored in the configuration account and any additional fees specific to the
/// connections used in the message. It first validates the input parameters, then queries the
/// fee for each connection specified in the `sources` list, and adds it to the protocol fee.
///
/// # Arguments
/// - `ctx`: The context of the solana program instruction.
/// - `nid`: A string representing the network ID for which the fee is being calculated.
/// - `is_rollback`: A boolean indicating whether a rollback is required, affecting the fee.
/// - `sources`: A vector of strings representing the source protocols involved in the transaction.
///
/// # Returns
/// - `Result<u64>`: Returns the total fee as a `u64` value if successful, otherwise returns
/// an error.
pub fn get_fee(
    ctx: Context<GetFeeCtx>,
    nid: String,
    is_rollback: bool,
    sources: Vec<String>,
) -> Result<u64> {
    if sources.is_empty() {
        return Err(XcallError::SourceProtocolsNotSpecified.into());
    }

    let mut data = vec![];
    let args = xcall_connection_type::GetFeeArgs {
        network_id: nid,
        response: is_rollback,
    };
    args.serialize(&mut data)?;

    let ix_data = helper::get_instruction_data(GET_FEE_IX, data);

    let mut connection_fee = ctx.accounts.config.protocol_fee;
    for (i, source) in sources.iter().enumerate() {
        let fee = connection::query_connection_fee(source, &ix_data, &ctx.remaining_accounts[i])?;
        if fee > 0 {
            connection_fee = connection_fee.checked_add(fee).expect("no overflow")
        }
    }

    Ok(connection_fee)
}

#[derive(Accounts)]
pub struct SetFeeHandlerCtx<'info> {
    /// The configuration account, which stores important settings for the program.
    /// This account is mutable because the fee handler of the protocol will be updated.
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

#[derive(Accounts)]
pub struct SetFeeCtx<'info> {
    /// The configuration account, which stores important settings for the program.
    /// This account is mutable because the fee handler of the protocol will be updated.
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

#[derive(Accounts)]
pub struct GetFeeCtx<'info> {
    /// The configuration account, which stores important settings for the program.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,
}
