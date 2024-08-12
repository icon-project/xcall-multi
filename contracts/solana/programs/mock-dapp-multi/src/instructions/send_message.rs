use anchor_lang::prelude::*;
use xcall_lib::{message::envelope::Envelope, network_address::*, xcall_dapp_type};

use crate::{helpers, xcall, state::*};

pub fn send_message<'info>(
    ctx: Context<'_, '_, '_, 'info, CallMessageCtx<'info>>,
    to: NetworkAddress,
    data: Vec<u8>,
    msg_type: u32,
    rollback: Vec<u8>,
) -> Result<()> {
    let message = helpers::process_message(msg_type as u8, data, rollback).unwrap();
    let (sources, destinations) = helpers::get_network_connections(&ctx)?;

    let envelope = Envelope::new(message, sources.clone(), destinations);
    let msg = rlp::encode(&envelope).to_vec();

    let ix_data = xcall::get_send_call_ix_data(msg, to)?;

    xcall::call_xcall_send_call(
        &ix_data,
        &ctx.accounts.config,
        &ctx.accounts.sender,
        &ctx.accounts.authority,
        &ctx.accounts.system_program,
        &ctx.remaining_accounts,
    )
}

pub fn add_connection(
    ctx: Context<AddConnectionCtx>,
    _network_id: String,
    src_endpoint: String,
    dst_endpoint: String,
) -> Result<()> {
    ctx.accounts
        .connection_account
        .connections
        .push(Connection {
            src_endpoint,
            dst_endpoint,
        });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeCtx<'info> {
    #[account(
        init,
        payer = sender,
        space = Config::MAX_SPACE, 
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = sender,
        space = Authority::MAX_SPACE,
        seeds = [xcall_dapp_type::DAPP_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(to: NetworkAddress )]
pub struct CallMessageCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [xcall_dapp_type::DAPP_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(
        mut,
        seeds = [Connections::SEED_PREFIX.as_bytes(), to.nid().as_bytes()],
        bump
    )]
    pub connections_account: Account<'info, Connections>,

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct AddConnectionCtx<'info> {
    #[account(
        init,
        payer = sender,
        space= Connections::MAX_SPACE,
        seeds=[Connections::SEED_PREFIX.as_bytes(), network_id.as_bytes()],
        bump
    )]
    pub connection_account: Account<'info, Connections>,

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}
