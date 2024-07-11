use anchor_lang::prelude::*;
use std::mem::size_of;

use xcall_lib::network_address::*;

pub mod error;
pub mod event;
pub mod helpers;
pub mod instructions;
pub mod state;

use crate::helpers::*;
use error::*;
use instructions::*;
use state::*;

declare_id!("Gj1bJC7rtUSYN8XZHpphDdTbqEGqsMPrRDGKhQJtFQyw");

#[program]

pub mod dapp_multi {

    use super::*;

    pub fn initialize(ctx: Context<InitializeCtx>, _xcall_address: Pubkey) -> Result<()> {
        ctx.accounts.config.set_inner(Config {
            xcall_address: _xcall_address,
            sn: 0,
        });
        Ok(())
    }

    pub fn send_call_message<'info>(
        ctx: Context<'_, '_, '_, 'info, CallMessageCtx<'info>>,
        to: NetworkAddress,
        data: Vec<u8>,
        msg_type: u32,
        rollback: Vec<u8>,
    ) -> Result<()> {
        let _ = instructions::send_message::send_message(ctx, to, data, msg_type, rollback);
        Ok(())
    }
    pub fn handle_call_message<'info>(
        _ctx: Context<'_, '_, '_, 'info, CallMessageCtx<'info>>,
        _from: NetworkAddress,
        _data: Vec<u8>,
    ) -> Result<()> {
        let _ = instructions::handle_message::handle_call_message();
        Ok(())
    }

    pub fn add_connection(
        ctx: Context<AddConnectionCtx>,
        _network_id: String,
        src_endpoint: String,
        dst_endpoint: String,
    ) -> Result<()> {
        let _ = instructions::send_message::add_connection(
            ctx,
            _network_id,
            src_endpoint,
            dst_endpoint,
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeCtx<'info> {
    #[account(init , payer = sender , space= 8 + size_of::<Config>() , seeds=[Config::SEED_PREFIX.as_bytes()] , bump )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}
