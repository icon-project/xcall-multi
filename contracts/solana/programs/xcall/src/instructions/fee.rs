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

pub fn get_fee(
    ctx: Context<GetFeeCtx>,
    nid: String,
    rollback: bool,
    sources: Vec<String>,
) -> Result<u64> {
    if sources.is_empty() {
        return Err(XcallError::ProtocolNotSpecified.into());
    }

    let mut data = vec![];
    let args = xcall_connection_type::GetFeeArgs {
        network_id: nid,
        response: rollback,
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
pub struct SetFeeCtx<'info> {
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
#[instruction(nid: String)]
pub struct GetFeeCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,
}
