use anchor_lang::prelude::*;

pub mod connection;
pub mod constants;
pub mod dapp;
pub mod error;
pub mod event;
pub mod helper;
pub mod instructions;
pub mod state;
pub mod types;

use instructions::*;

use types::message::CSMessageDecoded;
use xcall_lib::{
    network_address::NetworkAddress,
    query_account_type::{QueryAccountsPaginateResponse, QueryAccountsResponse},
};

declare_id!("9Fspq2pWy4uo8UtDtuxbQUas45W1p49t2gYPpxpfZmTy");

#[program]
pub mod xcall {
    use super::*;

    pub fn initialize(ctx: Context<ConfigCtx>, network_id: String) -> Result<()> {
        instructions::initialize(ctx, network_id)
    }

    pub fn set_admin(ctx: Context<SetAdminCtx>, account: Pubkey) -> Result<()> {
        instructions::set_admin(ctx, account)
    }

    pub fn set_protocol_fee(ctx: Context<SetFeeCtx>, fee: u64) -> Result<()> {
        instructions::set_protocol_fee(ctx, fee)
    }

    pub fn set_protocol_fee_handler(
        ctx: Context<SetFeeHandlerCtx>,
        fee_handler: Pubkey,
    ) -> Result<()> {
        instructions::set_protocol_fee_handler(ctx, fee_handler)
    }

    pub fn send_call<'info>(
        ctx: Context<'_, '_, '_, 'info, SendCallCtx<'info>>,
        envelope: Vec<u8>,
        to: NetworkAddress,
    ) -> Result<u128> {
        instructions::send_call(ctx, envelope, to)
    }

    #[allow(unused_variables)]
    pub fn handle_message(
        ctx: Context<HandleMessageCtx>,
        from_nid: String,
        msg: Vec<u8>,
        sequence_no: u128,
    ) -> Result<()> {
        instructions::handle_message(ctx, from_nid, msg)
    }

    pub fn handle_error<'info>(
        ctx: Context<'_, '_, '_, 'info, HandleErrorCtx<'info>>,
        sequence_no: u128,
    ) -> Result<()> {
        instructions::handle_error(ctx, sequence_no)
    }

    pub fn get_fee(
        ctx: Context<GetFeeCtx>,
        nid: String,
        rollback: bool,
        sources: Option<Vec<String>>,
    ) -> Result<u64> {
        instructions::get_fee(ctx, nid, rollback, sources.unwrap_or(vec![]))
    }

    pub fn get_admin(ctx: Context<GetConfigCtx>) -> Result<Pubkey> {
        Ok(ctx.accounts.config.admin)
    }

    pub fn get_protocol_fee(ctx: Context<GetConfigCtx>) -> Result<u64> {
        Ok(ctx.accounts.config.protocol_fee)
    }

    pub fn get_protocol_fee_handler(ctx: Context<GetConfigCtx>) -> Result<Pubkey> {
        Ok(ctx.accounts.config.fee_handler)
    }

    pub fn get_network_address(ctx: Context<GetConfigCtx>) -> Result<NetworkAddress> {
        Ok(NetworkAddress::new(
            &ctx.accounts.config.network_id,
            &id().to_string(),
        ))
    }

    #[allow(unused_variables)]
    pub fn get_default_connection(ctx: Context<GetConfigCtx>, nid: String) -> Result<Pubkey> {
        Ok(ctx.accounts.config.fee_handler)
    }

    #[allow(unused_variables)]
    pub fn decode_cs_message<'info>(
        ctx: Context<'_, '_, '_, 'info, EmptyContext<'info>>,
        message: Vec<u8>,
    ) -> Result<CSMessageDecoded> {
        instructions::decode_cs_message(message)
    }

    #[allow(unused_variables)]
    pub fn execute_call<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteCallCtx<'info>>,
        req_id: u128,
        data: Vec<u8>,
        from_nid: String,
    ) -> Result<()> {
        instructions::execute_call(ctx, req_id, data)
    }

    pub fn execute_rollback<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteRollbackCtx<'info>>,
        sn: u128,
    ) -> Result<()> {
        instructions::execute_rollback(ctx, sn)
    }

    pub fn query_execute_call_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryExecuteCallAccountsCtx<'info>>,
        req_id: u128,
        data: Vec<u8>,
        page: u8,
        limit: u8,
    ) -> Result<QueryAccountsPaginateResponse> {
        instructions::query_execute_call_accounts(ctx, req_id, data, page, limit)
    }

    #[allow(unused_variables)]
    pub fn query_execute_rollback_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryExecuteRollbackAccountsCtx<'info>>,
        sn: u128,
        page: u8,
        limit: u8,
    ) -> Result<QueryAccountsPaginateResponse> {
        instructions::query_execute_rollback_accounts(ctx, page, limit)
    }

    #[allow(unused_variables)]
    pub fn query_handle_message_accounts(
        ctx: Context<QueryHandleMessageAccountsCtx>,
        from_nid: String,
        msg: Vec<u8>,
        sequence_no: u128,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_handle_message_accounts(ctx, msg)
    }

    pub fn query_handle_error_accounts(
        ctx: Context<QueryHandleErrorAccountsCtx>,
        sequence_no: u128,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_handle_error_accounts(ctx, sequence_no)
    }
}