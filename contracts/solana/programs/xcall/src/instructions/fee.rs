use anchor_lang::prelude::*;
use xcall_lib::xcall_connection_msg::{self, GET_FEE_IX};

use crate::{connection, error::*, helper, send_message::is_reply, state::*};

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
    if !rollback && is_reply(&ctx.accounts.reply, &nid, &sources) {
        return Ok(0_u64);
    };

    let mut sources = sources;
    if sources.is_empty() {
        sources = vec![ctx.accounts.reply.key().to_string()]
    }

    let mut data = vec![];
    let args = xcall_connection_msg::GetFee {
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
        has_one = fee_handler @ XcallError::OnlyAdmin
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub fee_handler: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(nid: String)]
pub struct GetFeeCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), nid.as_bytes()],
        bump = default_connection.bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(
        seeds = [Reply::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub reply: Account<'info, Reply>,
}
