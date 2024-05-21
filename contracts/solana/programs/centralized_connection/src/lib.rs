use anchor_lang::{
    prelude::*,
    solana_program::{ program::invoke, system_instruction::transfer},
};
use std::mem::size_of;

use xcall::program::xcall;
use xcall::cpi::accounts::HandleMessageCtx;
use xcall::{self, ProxyReq, RollbackDataAccount,XCallState};




declare_id!("35XcNkLUyBHetw7CVZL6Gg9R1AhZgzS1ektLy1Yeiv8e");

//Instructions
#[program]
pub mod solana_centralized_connection {
    use super::*;

    pub fn initialize(
        _ctx: Context<InitializeCtx>,
        _relayer: Pubkey,
        _xcall: Pubkey,
    ) -> Result<()> {

        _ctx.accounts.centralized_connection_state.admin_address = _relayer;
        _ctx.accounts.centralized_connection_state.xcall_address = _xcall;
        Ok(())
    }

    pub fn set_admin(_ctx: Context<SetAdminCtx>, _admin: Pubkey) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.centralized_connection_state.admin_address,
            _ctx.accounts.user.key()
        );
        _ctx.accounts.centralized_connection_state.admin_address = _admin;
        Ok(())
    }

    pub fn get_admin(_ctx: Context<GetAdminCtx>) -> Result<Pubkey> {
        Ok(_ctx.accounts.centralized_connection_state.admin_address)
    }

    pub fn set_fee(
        _ctx: Context<SetFeeCtx>,
        _message_fee: u64,
        _response_fee: u64,
        _network: String,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.user.key(),
            _ctx.accounts.centralized_connection_state.admin_address, ErrorCode::SignerIsNotAuthority
        );
        _ctx.accounts.fees.message_fees = _message_fee;
        _ctx.accounts.fees.response_fees = _response_fee;
        _ctx.accounts.fees.network = _network;
        Ok(())
    }
    pub fn get_fee(_ctx: Context<GetFeeCtx>, _network: String, response: bool) -> Result<u64> {
        let message_fee = &mut _ctx.accounts.fees.message_fees.clone();

        if response == true {
            *message_fee += _ctx.accounts.fees.response_fees;
        }
        Ok(*message_fee)
    }

    pub fn send_message(
        _ctx: Context<SendMessageCtx>,
        _to: String,
        _svc: u128, // out of memory error occurs at testing when String or vec<> type is used here
        _sn: u128,
        _msg: Vec<u8>,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.user.key(),
            _ctx.accounts.centralized_connection_state.xcall_address
        );
        
        let mut fee: u64 = 0;
        if _sn > 0 {
            fee = _ctx.accounts.fees.total_fees(true);
        } else if _sn == 0 {
            fee = _ctx.accounts.fees.total_fees(false);
        }
        if _ctx.remaining_accounts[0].lamports() < fee {
            return Err(ErrorCode::InsufficientFee.into());
        }
        _ctx.accounts.centralized_connection_state.conn_sn += 1;

        invoke(
            &transfer(&_ctx.accounts.user.key(), &_ctx.accounts.centralized_connection_state.key(), fee),
            &[
                _ctx.accounts.user.to_account_info(),
                _ctx.accounts.centralized_connection_state.to_account_info(),
                _ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let event = MessageEvent {
            target_network: _to,
            sn: _sn,
            _msg,
        };
        emit!(event);
        Ok(())
    }

    

    pub fn recv_receipt(
        _ctx: Context<RecvReceiptCtx>,
        _src_network: String,
        _conn_sn: u128,
        _msg: Vec<u8>,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.centralized_connection_state.admin_address,
            _ctx.accounts.user.key()
        );
        if _ctx.accounts.receipt.receive_status == true {
            return Err(ErrorCode::DuplicateMessage.into());
        }
        _ctx.accounts.receipt.receive_status = true;

        let cpi_xcall_program = _ctx.accounts.xcall_program.to_account_info();
        
        let cpi_xcall_accounts = HandleMessageCtx {
            proxy_req : _ctx.accounts.proxy_req.to_account_info(),
            new_rollback_data: _ctx.accounts.new_rollback_data.to_account_info(),
            xcall_state : _ctx.accounts.xcall_state.to_account_info(),
            rollback_data : _ctx.accounts.rollback_data.to_account_info(),

        };
        let cpi_xcall_ctx = CpiContext::new(cpi_xcall_program, cpi_xcall_accounts);
        xcall::cpi::handle_message(cpi_xcall_ctx,_src_network, _msg );

        Ok(())
    }

    pub fn get_receipt(
        _ctx: Context<GetReceiptCtx>,
        _src_network: String,
        _conn_sn: u128,
    ) -> Result<bool> {
        return Ok(_ctx.accounts.receipt.receive_status);
    }


    // pub fn withdraw fees
}

// Instructure context structures
#[derive(Accounts)]
pub struct InitializeCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(init, payer= user, space = 8 + size_of::<CentralizedConnectionState>(), seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdminCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
}

#[derive(Accounts)]
pub struct GetAdminCtx<'info> {
    #[account( seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
}

#[derive(Accounts)]
// #[instruction(_network: String)]
pub struct SetFeeCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer= user, space = 8 + size_of::<FeesState>(), seeds = [b"fees"], bump)]
    // #[account(init, payer= user, space = 8 + size_of::<FeesState>(), seeds = [b"fees", _network.as_bytes().as_ref()], bump)]
    pub fees: Account<'info, FeesState>,
    #[account( seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
#[instruction(_network: String)]
pub struct GetFeeCtx<'info> {
    #[account(seeds = [b"fees"] , bump)]
    // #[account(seeds = [b"fees", _network.as_bytes().as_ref()] , bump)]
    pub fees: Account<'info, FeesState>,
}

#[derive(Accounts)]
#[instruction(_src_network: String, _conn_sn: u64)]

pub struct RecvReceiptCtx<'info> {
    #[account(init, space = 8 + size_of::<ReceiptState>() ,  payer = user , seeds = [b"receipt" , _src_network.as_bytes(), &_conn_sn.to_le_bytes()] , bump)]
    pub receipt: Account<'info, ReceiptState>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub xcall_program: Program<'info, xcall>,
    pub proxy_recp: Account<'info, ProxyReq>,
    pub new_rollback_data: Account<'info, RollbackDataAccount>,
    pub roll_back_account: Account<'info, RollbackDataAccount>,
    pub xcall_state: Account<'info, XCallState>,
    pub user: Signer<'info>,
    #[account( seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
}

#[derive(Accounts)]
#[instruction(_network: String, _conn_sn: u64)]
pub struct GetReceiptCtx<'info> {
    #[account(seeds = [b"receipt" , _network.as_bytes().as_ref(), &_conn_sn.to_le_bytes()] , bump)]
    pub receipt: Account<'info, ReceiptState>,
}

#[derive(Accounts)]
#[instruction(_sn: u128 , _to: String)]
pub struct SendMessageCtx<'info> {
    #[account(mut)]

    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account( mut , seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,

    // #[account(mut , seeds = [b"fees", _to.as_bytes().as_ref()] , bump)]
    #[account(mut , seeds = [b"fees"] , bump)]
    pub fees: Account<'info, FeesState>,
}

#[event]
pub struct MessageEvent {
    pub target_network: String,
    pub sn: u128,
    pub _msg: Vec<u8>,
}

// data structures
#[account]
#[derive(Default)]
pub struct CentralizedConnectionState {
    pub xcall_address: Pubkey,
    pub admin_address: Pubkey,
    pub conn_sn: u128,
}

#[account]
#[derive(Default)]

pub struct ReceiptState {
    src_network: String,
    conn_sn: u64,
    receive_status: bool,
}

#[account]
#[derive(Default)]
pub struct FeesState {
    network: String,
    message_fees: u64,
    response_fees: u64,
}

impl FeesState {
    pub fn new(network: String, message_fees: u64, response_fees: u64) -> Self {
        FeesState {
            network,
            message_fees,
            response_fees,
        }
    }

    pub fn total_fees(&self, response: bool) -> u64 {
        if response {
            self.message_fees + self.response_fees
        } else {
            self.message_fees
        }
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only Xcall can call sendMessage")]
    OnlyXcallCanCallSendMessage,
    #[msg("Fee is not sufficient")]
    InsufficientFee,
    #[msg("Invalid serial number")]
    DuplicateMessage,
    #[msg("Duplicate Message")]
    InvalidSerialNumber,
    #[msg("SignerIsNotAuthority")]
    SignerIsNotAuthority,
    #[msg("InsufficientPoints")]
    InsufficientPoints
}

