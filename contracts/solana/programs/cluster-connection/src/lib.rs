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

declare_id!("8oxnXrSmqWJqkb2spZk2uz1cegzPsLy6nJp9XwFhkMD5");

#[program]
pub mod centralized_connection {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, xcall: Pubkey, relayer: Pubkey) -> Result<()> {
        ctx.accounts
            .config
            .set_inner(Config::new(xcall, ctx.accounts.signer.key(), relayer, ctx.bumps.config));
        ctx.accounts
            .authority
            .set_inner(Authority::new(ctx.bumps.authority));

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
        ctx: Context<'_, '_, '_, 'info, ReceiveMessageWithSignatures<'info>>,
        src_network: String,
        conn_sn: u128,
        msg: Vec<u8>,
        sequence_no: u128,
        signatures: Vec<[u8; 65]>,
    ) -> Result<()> {
        helper::call_xcall_handle_message_with_signatures(ctx, src_network, msg, conn_sn, sequence_no, signatures)
    }

    pub fn revert_message<'info>(
        ctx: Context<'_, '_, '_, 'info, RevertMessage<'info>>,
        sequence_no: u128,
    ) -> Result<()> {
        helper::call_xcall_handle_error(ctx, sequence_no)
    }

    pub fn set_admin(ctx: Context<SetConfigItem>, account: Pubkey) -> Result<()> {
        let config = ctx.accounts.config.deref_mut();
        config.admin = account;

        Ok(())
    }

    pub fn set_relayer(ctx: Context<SetConfigItem>, address: Pubkey) -> Result<()> {
        let config = ctx.accounts.config.deref_mut();
        config.relayer = address;

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

    pub fn set_threshold(ctx: Context<SetConfigItem>, threshold: u8) -> Result<()> {
        if ctx.accounts.config.validators.len() < threshold as usize {
            return Err(error::ConnectionError::ValidatorsMustBeGreaterThanThreshold.into());
        }
        ctx.accounts.config.threshold = threshold;
        Ok(())
    }

    pub fn update_validators(ctx: Context<SetConfigItem>, validators: Vec<[u8; 65]>, threshold: u8) -> Result<()> {
        let mut unique_validators = validators.clone();
        unique_validators.sort();
        unique_validators.dedup();
        if unique_validators.len() < threshold as usize {
            return Err(error::ConnectionError::ValidatorsMustBeGreaterThanThreshold.into());
        }
        ctx.accounts.config.threshold = threshold;
        ctx.accounts.config.validators = unique_validators;
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
        **ctx.accounts.relayer.try_borrow_mut_lamports()? += fee;

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn query_send_message_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryAccountsCtx<'info>>,
        to: String,
        sn: i64,
        msg: Vec<u8>,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_send_message_accounts(ctx, to)
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
