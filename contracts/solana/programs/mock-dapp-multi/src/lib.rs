use anchor_lang::prelude::*;
use xcall_lib::{network_address::*, query_account_types::QueryAccountsResponse, xcall_dapp_msg};

pub mod error;
pub mod event;
pub mod helpers;
pub mod instructions;
pub mod state;
pub mod xcall;

use error::*;
use instructions::*;
use state::*;

declare_id!("88p1ScrjNFCqKEBwXqgKVoR8Zd4DRaQs3jq1t7oKDtdL");

#[program]
pub mod mock_dapp_multi {
    use super::*;

    pub fn initialize(ctx: Context<InitializeCtx>, _xcall_address: Pubkey) -> Result<()> {
        ctx.accounts.config.set_inner(Config {
            xcall_address: _xcall_address,
            sn: 0,
            bump: ctx.bumps.config,
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
        ctx: Context<'_, '_, '_, 'info, HandleCallMessageCtx<'info>>,
        from: NetworkAddress,
        data: Vec<u8>,
        protocols: Option<Vec<String>>,
    ) -> Result<xcall_dapp_msg::HandleCallMessageResponse> {
        instructions::handle_message::handle_call_message(ctx, from, data, protocols)
    }

    pub fn add_connection(
        ctx: Context<AddConnectionCtx>,
        network_id: String,
        src_endpoint: String,
        dst_endpoint: String,
    ) -> Result<()> {
        instructions::send_message::add_connection(ctx, network_id, src_endpoint, dst_endpoint)?;
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn query_handle_call_message_accounts(
        ctx: Context<QueryAccountsCtx>,
        from: NetworkAddress,
        data: Vec<u8>,
        protocols: Option<Vec<String>>,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_handle_call_message_accounts(ctx)
    }
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

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}
