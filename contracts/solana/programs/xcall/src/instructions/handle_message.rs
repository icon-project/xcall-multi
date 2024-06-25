use anchor_lang::{prelude::*, solana_program::hash};

use crate::{
    error::XcallError,
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
    sequence_no: u128,
) -> Result<()> {
    let config = &ctx.accounts.config;
    if config.network_id == from_nid.to_string() {
        return Err(XcallError::ProtocolMismatch.into());
    }

    let cs_message: CSMessage = message.try_into()?;
    match cs_message.message_type() {
        CSMessageType::CSMessageRequest => handle_request(ctx, from_nid, cs_message.payload()),
        CSMessageType::CSMessageResult => handle_result(ctx, cs_message.payload(), sequence_no),
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
    let source = ctx.accounts.signer.key();
    let source_valid = is_valid_source(
        &ctx.accounts.default_connection.key(),
        &source.to_string(),
        &req.protocols(),
    )?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    }

    if req.protocols().len() > 1 {
        let pending_requests = ctx
            .accounts
            .pending_request
            .as_mut()
            .ok_or(XcallError::PendingRequestsAccountNotSpecified)?;

        if !pending_requests.sources.contains(&source) {
            pending_requests.sources.push(source)
        }
        if pending_requests.sources.len() != req.protocols().len() {
            return Ok(());
        }
        // TODO: close account and refund lamports to sources
    }

    let req_id = ctx.accounts.config.get_next_req_id();

    emit!(event::CallMessage {
        from: req.from().to_string(),
        to: req.to().clone(),
        sn: req.sequence_no(),
        reqId: req_id,
        data: req.data()
    });

    req.hash_data();
    ctx.accounts
        .proxy_request
        .set_inner(ProxyRequest::new(req, source, ctx.bumps.proxy_request));

    Ok(())
}

pub fn handle_result(
    ctx: Context<HandleMessageCtx>,
    payload: &[u8],
    sequence_no: u128,
) -> Result<()> {
    let result: CSMessageResult = payload.try_into()?;

    let sender = ctx.accounts.signer.key();
    let rollback_account = ctx
        .accounts
        .rollback_account
        .as_mut()
        .ok_or(XcallError::CallRequestNotFound)?;

    let source_valid = is_valid_source(
        &ctx.accounts.default_connection.key(),
        &sender.to_string(),
        rollback_account.rollback.protocols(),
    )?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    }
    if sequence_no != result.sequence_no() {
        return Err(XcallError::InvalidMessageSequence.into());
    }

    if rollback_account.rollback.protocols().len() > 1 {
        let pending_responses = ctx
            .accounts
            .pending_response
            .as_mut()
            .ok_or(XcallError::PendingResponsesAccountNotSpecified)?;

        if !pending_responses.sources.contains(&sender) {
            pending_responses.sources.push(sender)
        }
        if pending_responses.sources.len() != rollback_account.rollback.protocols().len() {
            return Ok(());
        }
        // TODO: close account and refund lamports to sources
    }

    let response_code = result.response_code();

    emit!(event::ResponseMessage {
        code: response_code.clone().into(),
        sn: result.sequence_no()
    });

    match response_code {
        CSResponseType::CSResponseSuccess => {
            // TODO: close rollback account and refund lamports

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
        _ => {
            if rollback_account.rollback.rollback().len() < 1 {
                return Err(XcallError::NoRollbackData.into());
            }
            rollback_account.rollback.enable_rollback();

            emit!(event::RollbackMessage {
                sn: result.sequence_no()
            })
        }
    }

    Ok(())
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

    reply.hash_data();
    ctx.accounts.proxy_request.set_inner(ProxyRequest::new(
        reply.to_owned(),
        ctx.accounts.signer.key(),
        ctx.bumps.proxy_request,
    ));

    Ok(())
}

pub fn is_valid_source(
    default_connection: &Pubkey,
    sender: &String,
    protocols: &Vec<String>,
) -> Result<bool> {
    if protocols.contains(sender) {
        return Ok(true);
    }

    Ok(sender == &default_connection.to_string())
}

#[derive(Accounts)]
#[instruction(from_nid: String, message: Vec<u8>, sequence_no: u128)]
pub struct HandleMessageCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 640,
        seeds = ["req".as_bytes(), &hash::hash(&message).to_bytes()],
        bump
    )]
    pub pending_request: Option<Account<'info, PendingRequest>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 640,
        seeds = ["res".as_bytes(), &hash::hash(&message).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1,
        seeds = ["success".as_bytes(), sequence_no.to_string().as_bytes()],
        bump
    )]
    pub successful_response: Option<Account<'info, SuccessfulResponse>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ProxyRequest::SIZE,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), (config.last_req_id + 1).to_string().as_bytes()],
        bump
    )]
    pub proxy_request: Account<'info, ProxyRequest>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), from_nid.as_bytes()],
        bump = default_connection.bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), sequence_no.to_string().as_bytes()],
        bump
    )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
