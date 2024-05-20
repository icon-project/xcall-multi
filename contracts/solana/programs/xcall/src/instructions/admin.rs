use anchor_lang::prelude::*;
use crate::{error::ErrorCode::*, CSMessageRequest};

pub const XCALL_SEED: &str = "xcall";
pub const REPLY_DATA: &str = "xcall-reply-data";


#[account]
pub struct ReplyData {
    pub call_reply: Vec<u8>,
    pub reply_state: CSMessageRequest,
}

#[account]
#[derive(InitSpace)]
pub struct XCallState {
    pub xcall_admin: Pubkey,
    pub fee_handler: Pubkey,
    pub fee: u64,
    pub sequence_number: u128,
    pub request_id: u128,
    #[max_len(25)]
    pub network_id: String,
}

#[derive(Accounts)]
pub struct XCallStateCtx<'info> {
    #[account(
        init,
        seeds = [XCALL_SEED.as_bytes()],
        bump,
        payer = owner,
        space = 8 + XCallState::INIT_SPACE,
    )]
    pub xcall_state: Account<'info, XCallState>,
    #[account(
        init,
        seeds = [REPLY_DATA.as_bytes()],
        bump,
        payer = owner,
        space = 8 + 1024,
    )]
    pub reply_state: Account<'info ,ReplyData>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct XCallStateUpdateCtx<'info> {
    #[account(mut)]
    pub xcall_state: Account<'info, XCallState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}


pub fn initialize(ctx: Context<XCallStateCtx>, network_id: String) -> Result<()> {
    let state = &mut ctx.accounts.xcall_state;

    state.xcall_admin = ctx.accounts.owner.key();
    state.fee_handler = ctx.accounts.owner.key();
    state.fee = 0;
    state.network_id = network_id;

    Ok(())
}

pub fn set_admin(ctx: Context<XCallStateUpdateCtx>, new_admin: Pubkey) -> Result<()> {
    let state = &mut ctx.accounts.xcall_state;
    require_keys_eq!(ctx.accounts.owner.key(), state.xcall_admin);

    state.xcall_admin = new_admin;
    Ok(())
}

pub fn set_fee_handler(ctx: Context<XCallStateUpdateCtx>, fee_handler: Pubkey) -> Result<()> {
    let state = &mut ctx.accounts.xcall_state;
    require_keys_eq!(ctx.accounts.owner.key(), state.xcall_admin);

    state.fee_handler = fee_handler;
    Ok(())
}

pub fn set_fee(ctx: Context<XCallStateUpdateCtx>, fee: u64) -> Result<()> {
    let state = &mut ctx.accounts.xcall_state;
    require_keys_eq!(
        ctx.accounts.owner.key(),
        state.xcall_admin,
        Unauthorized
    );

    state.fee = fee;
    Ok(())
}
