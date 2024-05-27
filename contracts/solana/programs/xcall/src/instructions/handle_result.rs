use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::{error::ErrorCode, transfer_lamports, CSMessageResult, CallServiceResponseType, HandleMessageCtx};

pub fn handle_result(ctx: Context<HandleMessageCtx>, data: Vec<u8>) -> Result<()> {
    let result = CSMessageResult::unmarshal_from(&data).unwrap();
    let res_sn = result.sequence_no;

    let rollback_data = &mut ctx.accounts.rollback_data;

    if rollback_data.protocols.len() > 1 {
        // mutable reference for pending responses
        {
            let pending_responses = &mut ctx.accounts.pending_responses;
            pending_responses
                .sender
                .push(ctx.accounts.user.key().to_string());
            pending_responses.status.push(true);
            pending_responses.result_sn = res_sn;
        }

        let pending_responses = &ctx.accounts.pending_responses;

        for i in 0..rollback_data.protocols.len() {
            // if responses not received yet
            if !pending_responses
                .sender
                .contains(&rollback_data.protocols[i])
            {
                return Ok(());
            }

            // all responses received, lets destroy pending responses for this sn
            // for this, send all funds in pending responses account to fee handler
            transfer_lamports(
                &ctx.accounts.pending_responses.to_account_info(),
                &ctx.accounts.fee_handler.to_account_info(),
            )
            .unwrap();

            ctx.accounts
                .pending_responses
                .close(ctx.accounts.user.to_account_info())?;
        }
    } else if rollback_data.protocols.len() == 1 {
        let source = Pubkey::from_str(&rollback_data.protocols[0]).unwrap();
        require_keys_eq!(source, ctx.accounts.user.key(), ErrorCode::Unauthorized);
    }
    // default connection not handled

    emit!(ResponseMessage {
        sn: res_sn,
        code: result.response_code
    });

    match CallServiceResponseType::from(result.response_code) {
        CallServiceResponseType::CallServiceResponseSuccess => {
            // delete call request, transfer all funds from rollback_data to fee handler
            transfer_lamports(
                &ctx.accounts.rollback_data.to_account_info(),
                &ctx.accounts.fee_handler.to_account_info(),
            )
            .unwrap();

            ctx.accounts
                .rollback_data
                .close(ctx.accounts.user.to_account_info())?;
        }
        CallServiceResponseType::CallServiceResponseFailure => {
            require_gte!(rollback_data.rollback.len(), 0);
            rollback_data.enabled = true;
            rollback_data.from = rollback_data.from;
            rollback_data.to = rollback_data.to.clone();
            rollback_data.protocols = rollback_data.protocols.clone();
            rollback_data.rollback = rollback_data.rollback.clone();

            emit!(RollbackMessage { sn: res_sn });
        }
    }

    Ok(())
}

#[event]
pub struct ResponseMessage {
    pub sn: u128,
    pub code: u8,
}
#[event]
pub struct CallMessage {
    pub from: String,
    pub to: String,
    pub sn: u128,
    pub req_id: u128,
    pub data: Vec<u8>,
}

#[event]
pub struct RollbackMessage {
    pub sn: u128,
}
