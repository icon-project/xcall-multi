use std::ops::DerefMut;

use anchor_lang::prelude::*;

pub mod constants;
pub mod contexts;
pub mod error;
pub mod event;
pub mod helper;
pub mod instructions;
pub mod state;

use contexts::*;
use instructions::*;
use state::*;

use xcall_lib::query_account_type::{QueryAccountsPaginateResponse, QueryAccountsResponse};

declare_id!("4vfkXyxMxptmREF3RaFKUwnPRuqsXJJeUFzpCjPSSVMb");

#[program]
pub mod centralized_connection {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, xcall: Pubkey, admin: Pubkey) -> Result<()> {
        ctx.accounts
            .config
            .set_inner(Config::new(xcall, admin, ctx.bumps.config));

        Ok(())
    }

    pub fn send_message(
        ctx: Context<SendMessage>,
        to: String,
        sn: i64,
        msg: Vec<u8>,
    ) -> Result<()> {
        let next_conn_sn = ctx.accounts.config.get_next_conn_sn()?;

        let mut fee = 0;
        if sn >= 0 {
            fee = ctx.accounts.network_fee.get(sn > 0)?;
        }

        if fee > 0 {
            helper::transfer_lamports(
                &ctx.accounts.signer,
                &ctx.accounts.config.to_account_info(),
                &ctx.accounts.system_program,
                fee,
            )?
        }

        emit!(event::SendMessage {
            targetNetwork: to,
            connSn: next_conn_sn,
            msg: msg
        });

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn recv_message<'info>(
        ctx: Context<'_, '_, '_, 'info, RecvMessage<'info>>,
        src_network: String,
        conn_sn: u128,
        msg: Vec<u8>,
        sequence_no: u128,
    ) -> Result<()> {
        helper::call_xcall_handle_message(ctx, src_network, msg, sequence_no)
    }

    pub fn revert_message<'info>(
        ctx: Context<'_, '_, '_, 'info, RevertMessage<'info>>,
        sequence_no: u128,
    ) -> Result<()> {
        helper::call_xcall_handle_error(ctx, sequence_no)
    }

    pub fn set_admin(ctx: Context<SetAdmin>, account: Pubkey) -> Result<()> {
        let config = ctx.accounts.config.deref_mut();
        config.admin = account;

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_fee(
        ctx: Context<SetFee>,
        network_id: String,
        message_fee: u64,
        response_fee: u64,
    ) -> Result<()> {
        ctx.accounts.network_fee.set_inner(NetworkFee::new(
            message_fee,
            response_fee,
            ctx.bumps.network_fee,
        ));

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn get_fee(ctx: Context<GetFee>, network_id: String, response: bool) -> Result<u64> {
        ctx.accounts.network_fee.get(response)
    }

    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        let config = ctx.accounts.config.to_account_info();
        let fee = ctx.accounts.config.get_claimable_fees(&config)?;

        **config.try_borrow_mut_lamports()? -= fee;
        **ctx.accounts.admin.try_borrow_mut_lamports()? += fee;

        Ok(())
    }

    pub fn query_send_message_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryAccountsCtx<'info>>,
        dst_network: String,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_send_message_accounts(ctx, dst_network)
    }

    pub fn query_recv_message_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryAccountsCtx<'info>>,
        src_network: String,
        conn_sn: u128,
        msg: Vec<u8>,
        sequence_no: u128,
        page: u8,
        limit: u8,
    ) -> Result<QueryAccountsPaginateResponse> {
        instructions::query_recv_message_accounts(
            ctx,
            src_network,
            conn_sn,
            msg,
            sequence_no,
            page,
            limit,
        )
    }

    pub fn query_revert_message_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryAccountsCtx<'info>>,
        sequence_no: u128,
        page: u8,
        limit: u8,
    ) -> Result<QueryAccountsPaginateResponse> {
        instructions::query_revert_message_accounts(ctx, sequence_no, page, limit)
    }
}
