use anchor_lang::prelude::*;

pub mod assertion;
pub mod constants;
pub mod error;
pub mod event;
pub mod instructions;
pub mod state;
pub mod types;

use instructions::*;
use state::*;

use xcall_lib::network_address::NetworkAddress;

declare_id!("DL5ULXfYtnE5m8swfivfxtaPM4y3bcsDphseZkWFXgft");

#[program]
pub mod xcall {
    use super::*;

    pub fn initialize(ctx: Context<ConfigCtx>, network_id: String) -> Result<()> {
        ctx.accounts
            .config
            .set_inner(Config::new(ctx.accounts.signer.key(), network_id));

        ctx.accounts.reply.new();

        Ok(())
    }

    pub fn set_admin(ctx: Context<UpdateConfigCtx>, account: Pubkey) -> Result<()> {
        ctx.accounts
            .config
            .ensure_admin(ctx.accounts.signer.key())?;

        ctx.accounts.config.set_admin(account);

        Ok(())
    }

    pub fn set_protocol_fee(ctx: Context<UpdateConfigCtx>, fee: u64) -> Result<()> {
        ctx.accounts
            .config
            .ensure_fee_handler(ctx.accounts.signer.key())?;

        ctx.accounts.config.set_protocol_fee(fee);

        Ok(())
    }

    pub fn set_protocol_fee_handler(
        ctx: Context<UpdateConfigCtx>,
        fee_handler: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .config
            .ensure_admin(ctx.accounts.signer.key())?;

        ctx.accounts.config.set_fee_handler(fee_handler);

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_default_connection(
        ctx: Context<DefaultConnectionCtx>,
        network_id: String,
        connection: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .config
            .ensure_admin(ctx.accounts.signer.key())?;

        ctx.accounts
            .default_connection
            .set(connection, ctx.bumps.default_connection);

        Ok(())
    }

    pub fn send_call<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SendCallCtx<'info>>,
        envelope: Vec<u8>,
        to: NetworkAddress,
    ) -> Result<u128> {
        instructions::send_call(ctx, envelope, to)
    }

    #[allow(unused_variables)]
    pub fn handle_message(
        ctx: Context<HandleMessageCtx>,
        from_nid: String,
        message: Vec<u8>,
        sequence_no: u128,
    ) -> Result<()> {
        instructions::handle_message(ctx, from_nid, message, sequence_no)
    }

    pub fn execute_call<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ExecuteCallCtx<'info>>,
        req_id: u128,
        data: Vec<u8>,
        nid: String
    ) -> Result<()> {
        instructions::execute_call(ctx, req_id, data,nid)
    }

    pub fn execute_rollback<'a, 'b, 'c, 'info>(
        ctx : Context<'a, 'b, 'c, 'info, ExecuteRollbackCtx<'info>>,
        sn : u128
    ) -> Result<()> {
        instructions::execute_rollback(ctx,sn)
    }
}
