use anchor_lang::prelude::*;
use xcall_lib::{network_address::*, query_account_type::QueryAccountsResponse, xcall_dapp_type};

pub mod constants;
pub mod error;
pub mod event;
pub mod helpers;
pub mod instructions;
pub mod state;
pub mod xcall;

use error::*;
use instructions::*;
use state::*;

declare_id!("8qUFXfNw2VHTthqBNuTQ8d1QMQKEr5NbhPbXjkryWzmv");

#[program]
pub mod mock_dapp_multi {
    use super::*;

    pub fn initialize(ctx: Context<InitializeCtx>, xcall_address: Pubkey) -> Result<()> {
        ctx.accounts.config.set_inner(Config {
            xcall_address,
            sn: 0,
            bump: ctx.bumps.config,
        });
        ctx.accounts.authority.set_inner(Authority {
            bump: ctx.bumps.authority,
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
        protocols: Vec<String>,
    ) -> Result<xcall_dapp_type::HandleCallMessageResponse> {
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

    pub fn execute_forced_rollback<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteForcedRollbackCtx<'info>>,
        req_id: u128,
    ) -> Result<()> {
        instructions::execute_forced_rollback(ctx, req_id)
    }

    #[allow(unused_variables)]
    pub fn query_handle_call_message_accounts(
        ctx: Context<QueryAccountsCtx>,
        from: NetworkAddress,
        data: Vec<u8>,
        protocols: Vec<String>,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_handle_call_message_accounts(ctx)
    }
}
