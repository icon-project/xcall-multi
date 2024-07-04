use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke},
    },
};
use xcall_lib::xcall_connection_msg;

use crate::{error::*, helper, send_message::is_reply, state::*};

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

    let ix_data = helper::get_instruction_data("get_fee", data);

    let mut connection_fee = ctx.accounts.config.protocol_fee;
    for (i, source) in sources.iter().enumerate() {
        let fee = query_connection_fee(source, &ix_data, &ctx.remaining_accounts[i])?;
        if fee > 0 {
            connection_fee = connection_fee.checked_add(fee).expect("no overflow")
        }
    }

    Ok(connection_fee)
}

pub fn query_connection_fee<'info>(
    source: &String,
    ix_data: &Vec<u8>,
    network_fee_account: &AccountInfo,
) -> Result<u64> {
    let connection = Pubkey::from_str(&source)
        .ok()
        .ok_or(XcallError::InvalidSource)?;

    let account_metas = vec![AccountMeta::new(network_fee_account.key(), false)];
    let account_infos = vec![network_fee_account.to_account_info()];

    let ix = Instruction {
        program_id: connection,
        accounts: account_metas,
        data: ix_data.clone(),
    };

    invoke(&ix, &account_infos)?;

    let (_, fee) = get_return_data().unwrap();
    let fee = u64::deserialize(&mut fee.as_ref())?;

    Ok(fee)
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
