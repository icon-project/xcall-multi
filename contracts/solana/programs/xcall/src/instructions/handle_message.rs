use anchor_lang::{prelude::*, solana_program::hash};

use crate::{
    error::*,
    event,
    state::*,
    types::{
        message::{CSMessage, CSMessageType},
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
    },
};

pub fn handle_message(
    ctx: Context<HandleMessageCtx>,
    from_nid: String,
    message: Vec<u8>,
) -> Result<()> {
    let config = &ctx.accounts.config;
    if config.network_id == from_nid.to_string() {
        return Err(XcallError::ProtocolMismatch.into());
    }

    let cs_message: CSMessage = message.try_into()?;
    match cs_message.message_type() {
        CSMessageType::CSMessageRequest => handle_request(ctx, from_nid, cs_message.payload()),
        CSMessageType::CSMessageResult => handle_result(ctx, cs_message.payload()),
    }
}

pub fn handle_request(
    ctx: Context<HandleMessageCtx>,
    from_nid: String,
    payload: &[u8],
) -> Result<()> {
    let mut req: CSMessageRequest = payload.try_into()?;

    let (src_nid, _) = req.from().parse_network_address();
    if src_nid != from_nid {
        return Err(XcallError::ProtocolMismatch.into());
    }
    let source = ctx.accounts.connection.owner.to_owned();
    let source_valid = is_valid_source(&source.to_string(), &req.protocols())?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    }

    if req.protocols().len() > 1 {
        let pending_request = ctx
            .accounts
            .pending_request
            .as_mut()
            .ok_or(XcallError::PendingRequestAccountNotSpecified)?;

        if !pending_request.sources.contains(&source) {
            pending_request.sources.push(source)
        }
        if pending_request.sources.len() != req.protocols().len() {
            return Ok(());
        }
        pending_request.close(ctx.accounts.admin.clone())?;
    }

    let req_id = ctx.accounts.config.get_next_req_id();

    emit!(event::CallMessage {
        from: req.from().to_string(),
        to: req.to().clone(),
        sn: req.sequence_no(),
        reqId: req_id,
        data: req.data()
    });

    let proxy_request = ctx
        .accounts
        .proxy_request
        .as_deref_mut()
        .ok_or(XcallError::ProxyRequestAccountNotSpecified)?;

    req.hash_data();
    proxy_request.set(req, ctx.bumps.proxy_request.unwrap());

    Ok(())
}

pub fn handle_result(ctx: Context<HandleMessageCtx>, payload: &[u8]) -> Result<()> {
    let result: CSMessageResult = payload.try_into()?;

    let sender = ctx.accounts.connection.owner.key();
    let rollback_account = ctx
        .accounts
        .rollback_account
        .as_mut()
        .ok_or(XcallError::CallRequestNotFound)?;

    validate_source_and_pending_response(
        sender,
        rollback_account.rollback.protocols(),
        &mut ctx.accounts.pending_response,
        &ctx.accounts.admin,
    )?;

    let response_code = result.response_code();

    emit!(event::ResponseMessage {
        code: response_code.clone().into(),
        sn: result.sequence_no()
    });

    match response_code {
        CSResponseType::CSResponseSuccess => {
            rollback_account.close(ctx.accounts.admin.clone())?;

            let success_res = ctx
                .accounts
                .successful_response
                .as_mut()
                .ok_or(XcallError::SuccessfulResponseAccountNotSpecified)?;

            success_res.success = true;

            if result.message().is_some() {
                let reply = &mut result.message().unwrap();
                handle_reply(ctx, reply)?;
            }
        }
        _ => handle_rollback(rollback_account, result.sequence_no())?,
    }

    Ok(())
}

pub fn handle_error(ctx: Context<HandleErrorCtx>, sequence_no: u128) -> Result<()> {
    let sender = ctx.accounts.connection.owner.key();
    let rollback_account = &mut ctx.accounts.rollback_account;

    validate_source_and_pending_response(
        sender,
        rollback_account.rollback.protocols(),
        &mut ctx.accounts.pending_response,
        &ctx.accounts.admin,
    )?;

    emit!(event::ResponseMessage {
        code: CSResponseType::CSResponseFailure.into(),
        sn: sequence_no
    });

    handle_rollback(rollback_account, sequence_no)
}

pub fn handle_reply(ctx: Context<HandleMessageCtx>, reply: &mut CSMessageRequest) -> Result<()> {
    if let Some(rollback_account) = &ctx.accounts.rollback_account {
        if rollback_account.rollback.to().nid() != reply.from().nid() {
            return Err(XcallError::InvalidReplyReceived.into());
        }
    }

    let req_id = ctx.accounts.config.get_next_req_id();

    emit!(event::CallMessage {
        from: reply.from().to_string(),
        to: reply.to().clone(),
        sn: reply.sequence_no(),
        reqId: req_id,
        data: reply.data()
    });

    let proxy_request = ctx
        .accounts
        .proxy_request
        .as_deref_mut()
        .ok_or(XcallError::ProxyRequestAccountNotSpecified)?;

    reply.hash_data();
    proxy_request.set(reply.to_owned(), ctx.bumps.proxy_request.unwrap());

    Ok(())
}

pub fn validate_source_and_pending_response<'info>(
    sender: Pubkey,
    protocols: &Vec<String>,
    pending_response: &mut Option<Account<'info, PendingResponse>>,
    admin: &AccountInfo<'info>,
) -> Result<()> {
    let source_valid = is_valid_source(&sender.to_string(), protocols)?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    };

    if protocols.len() > 1 {
        let pending_response = pending_response
            .as_mut()
            .ok_or(XcallError::PendingResponseAccountNotSpecified)?;

        if !pending_response.sources.contains(&sender) {
            pending_response.sources.push(sender)
        }
        if pending_response.sources.len() != protocols.len() {
            return Ok(());
        }
        pending_response.close(admin.to_owned())?;
    }

    Ok(())
}

pub fn handle_rollback(
    rollback_account: &mut Account<RollbackAccount>,
    sequence_no: u128,
) -> Result<()> {
    if rollback_account.rollback.rollback().len() < 1 {
        return Err(XcallError::NoRollbackData.into());
    }
    rollback_account.rollback.enable_rollback();

    emit!(event::RollbackMessage { sn: sequence_no });

    Ok(())
}

#[inline(never)]
pub fn is_valid_source(sender: &String, protocols: &Vec<String>) -> Result<bool> {
    if protocols.contains(sender) {
        return Ok(true);
    }

    Ok(false)
}

#[derive(Accounts)]
#[instruction(from_nid: String, msg: Vec<u8>, sequence_no: u128)]
pub struct HandleMessageCtx<'info> {
    pub connection: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin @ XcallError::InvalidAdminKey
    )]
    pub config: Box<Account<'info, Config>>,

    /// CHECK: this is safe because we are verifying if the passed account is admin or not
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ProxyRequest::SIZE,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), &(config.last_req_id + 1).to_be_bytes()],
        bump
    )]
    pub proxy_request: Option<Box<Account<'info, ProxyRequest>>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = PendingRequest::SIZE,
        seeds = [PendingRequest::SEED_PREFIX.as_bytes(), &hash::hash(&msg).to_bytes()],
        bump,
    )]
    pub pending_request: Option<Box<Account<'info, PendingRequest>>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = PendingResponse::SIZE,
        seeds = [PendingResponse::SEED_PREFIX.as_bytes(), &hash::hash(&msg).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = SuccessfulResponse::SIZE,
        seeds = [SuccessfulResponse::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub successful_response: Option<Box<Account<'info, SuccessfulResponse>>>,

    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Option<Box<Account<'info, RollbackAccount>>>,
}

#[derive(Accounts)]
#[instruction(sequence_no: u128)]
pub struct HandleErrorCtx<'info> {
    pub connection: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
        has_one = admin @ XcallError::InvalidAdminKey
    )]
    pub config: Account<'info, Config>,

    /// CHECK: this is safe because we are verifying if the passed account is admin or not
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Account<'info, RollbackAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        space = PendingResponse::SIZE,
        seeds = [PendingResponse::SEED_PREFIX.as_bytes(), &hash::hash(&CSMessageResult::new(sequence_no, CSResponseType::CSResponseFailure, None).as_bytes()).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,
}
