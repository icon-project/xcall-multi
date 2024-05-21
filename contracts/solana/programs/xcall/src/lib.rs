pub mod error;
pub mod instructions;
pub mod structs;

use anchor_lang::prelude::*;
use instructions::*;
use structs::*;

declare_id!("BfG9Pd1SRVWerTQxXuuti6Dq56hGY8TNFXdYLxBP2Hgr");

#[program]
pub mod xcall {
    use super::*;

    pub fn initialize(ctx: Context<XCallStateCtx>, network_id: String) -> Result<()> {
        instructions::initialize(ctx, network_id)
    }

    pub fn set_admin(ctx: Context<XCallStateUpdateCtx>, new_admin: Pubkey) -> Result<()> {
        instructions::set_admin(ctx, new_admin)
    }

    pub fn set_fee_handler(ctx: Context<XCallStateUpdateCtx>, fee_handler: Pubkey) -> Result<()> {
        instructions::set_fee_handler(ctx, fee_handler)
    }

    pub fn set_fee(ctx: Context<XCallStateUpdateCtx>, fee: u64) -> Result<()> {
        instructions::set_fee(ctx, fee)
    }

    pub fn send_message<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SendMessageCtx<'info>>,
        to: String,
        msg: Vec<u8>,
    ) -> Result<()> {
        instructions::send_message(ctx, to, msg)
    }

    pub fn send_message_with_rollback<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SendMessageWithRollbackCtx<'info>>,
        to: String,
        msg: Vec<u8>,
    ) -> Result<()> {
        instructions::send_message_with_rollback(ctx, to, msg)
    }

    pub fn handle_message(
        ctx: Context<HandleMessageCtx>,
        from_nid: String,
        msg: Vec<u8>,
    ) -> Result<()> {
        let cs_msg = CSMessage::unmarshal_from(&msg).unwrap();
        let state_info = &ctx.accounts.xcall_state;
        require_eq!(from_nid, state_info.network_id.clone());

        let msg_type = CSMessageType::as_type(cs_msg.msg_type);
        match msg_type {
            CSMessageType::CSMessageRequest => handle_request(ctx, from_nid, cs_msg.payload),
            CSMessageType::CSMessageResult => handle_result(ctx, cs_msg.payload),
        }
    }

    pub fn execute_call(ctx: Context<ExecuteCall>, req_id: u128, msg: Vec<u8>) -> Result<()> {
        instructions::execute_call(ctx, req_id, msg)
    }

    pub fn handle_message(ctx: Context<HandleMessageCtx>, from_nid: String, payload: Vec<u8>) -> Result<()> {
        instructions::handle_request(ctx, from_nid, payload)
    }

    pub fn execute_rollback(ctx: Context<Rollback>, sn: u128) -> Result<()> {
        instructions::execute_rollback(ctx, sn)
    }
}
