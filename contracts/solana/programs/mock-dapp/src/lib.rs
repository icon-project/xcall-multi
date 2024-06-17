use anchor_lang::prelude::*;
use std::mem::size_of;
use std::vec;

use xcall::instructions::admin::XCallState;
use xcall::program::Xcall;

// use centralized_connection::instructions::admin::CentralizedConnectionState;
// use centralized_connection::program::centralized_connection;

declare_id!("8Q4FvsHCWK68EzYtsstdFYwUL1SHCiuLPRDJk1gaKiQ8");

#[program]
pub mod mock {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _call_service: Pubkey) -> Result<()> {
        let dapp_state = ctx.accounts.dapp_state;
        dapp_state.call_svc = _call_service;

        // todo!("callSvcNetAddr = ICallService(callSvc).getNetworkAddress();")

        Ok(())
    }

    pub fn send_message(
        ctx: Context<SendMessageCtx>,
        _to: String,
        _data: Vec<u8>,
        _roll_back: Vec<u8>,
    ) -> Result<()> {
        let last_id = ctx.accounts.dapp_state.last_id;

        if (_roll_back.len() > 0) {
            ctx.accounts.dapp_state.last_id = last_id + 1;
            // bytes memory encodedRd = abi.encode(id, _rollback);

            // uint256 sn = ICallService(callSvc).sendCallMessage{value:msg.value}(
            //     _to,
            //     _data,
            //     encodedRd
            // );
            // rollbacks[id] = RollbackData(id, _rollback, sn);
        } else {
            //    ICallService(callSvc).sendCallMessage{value:msg.value}(
            //     _to,
            //     _data,
            //     _rollback
            // );
        }
        Ok(());
    }

    pub fn handle_call_message(
        ctx: Context<HandleCallMessageCtx>,
        _from: String,
        _data: Vec<u8>,
    ) -> Result<()> {
        // todo onlycallservice
        let call_svc_net_addr = &ctx.accounts.dapp_state.call_svc_net_addr;
        if call_svc_net_addr == _from {
            // (uint256 id, bytes memory received) = abi.decode(_data, (uint256, bytes));
            let id = 0;
            let received = "message";

            //need to pass this id to CTX struct to get its corresponding rollback account
            // RollbackData memory stored = rollbacks[id];

            require(
                compareTo(string(received), string(stored.rollback)),
                "rollbackData mismatch",
            );
            // delete rollbacks[id]; // cleanup
            let emitData = RollbackDataReceived(_from, stored.ssn, received);

            emit!(emitData)
        } else {
            // normal message delivery
            let emitData = MessageReceived(_from, _data);
            emit!(emitData);
        }
        Ok(());
    }
}

#[derive(Accounts)]
pub struct HandleCallMessageCtx<'info> {
    #[account(seeds = [ b"dappstate" , ], bump)]
    dapp_state: Account<'info, DappStateStruct>,
}

#[derive(Accounts)]
pub struct SendMessageCtx<'info> {
    // #[account(
    //     init_if_needed,
    //     payer = sender,
    //     space = 8 + 512,
    //     seeds=[b"rollback_data_state"],
    //     bump
    // )]
    // pub rollback_data_state: Account<'info, RollbackDataState>,

    // #[account(mut, seeds = [b"xcall"], bump)]
    // pub xcall_state: Box<Account<'info, XCallState>>,

    // #[account(mut)]
    // pub sender: Signer<'info>,
    // #[account(mut)]
    // pub reply_data: Box<Account<'info, ReplyData>>,
    // /// CHECK: to transfer protocol fee
    // #[account(mut)]
    // pub fee_handler: AccountInfo<'info>,
    // pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init , payer= sender , space = 8+ size_of::<DappStateStruct>(), seeds = [ b"dappstate" , ], bump)]
    dapp_state: Account<'info, DappStateStruct>,
    xcall_state: Account<'info, XCallState>,
    #[account(mut)]
    sender: Signer<'info>,
    system_program: Program<'info, System>,
}

pub struct DappStateStruct {
    pub call_svc: Pubkey,
    pub call_svc_net_addr: String,
    pub last_id: u128,
}

pub struct RollbackData {
    pub id: u128,
    pub rollback: Vec<u8>,
    pub ssn: u128,
}

#[event]
pub struct MessageReceivedEvent {
    pub from: String,
    pub data: Box<Vec<u8>>,
}
#[event]
pub struct RollbackDataReceivedEvent {
    pub from: String,
    pub ssn: u128,
    pub rollback: Box<Vec<u8>>,
}
