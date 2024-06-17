use std::ops::DerefMut;

use anchor_lang::prelude::*;
use xcall_lib::network_address::NetId;

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
    from_nid: NetId,
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
    from_nid: NetId,
    payload: &[u8],
) -> Result<()> {
    let mut req: CSMessageRequest = payload.try_into()?;

    let (src_nid, _) = req.from().parse_network_address();
    if src_nid != from_nid {
        return Err(XcallError::ProtocolMismatch.into());
    }
    let default_connection = ctx.accounts.default_connection.key().to_string();
    let source = ctx.accounts.signer.key().to_string();
    let source_valid = is_valid_source(&default_connection, &source, &req.protocols())?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    }

    if req.protocols().len() > 1 {
        let pending_requests = ctx
            .accounts
            .pending_requests
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
    ctx.accounts.proxy_request.set_inner(ProxyRequest {
        req: req.clone(),
        bump: ctx.bumps.proxy_request,
    });

    Ok(())
}

pub fn handle_result(ctx: Context<HandleMessageCtx>, payload: &[u8]) -> Result<()> {
    let result: CSMessageResult = payload.try_into()?;

    let default_connection = ctx.accounts.default_connection.key().to_string();
    let sender = ctx.accounts.signer.key().to_string();
    let rollback = &mut ctx.accounts.rollback_accunt.deref_mut().rollback;

    let source_valid = is_valid_source(&default_connection, &sender, rollback.protocols())?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    }

    if rollback.protocols().len() > 1 {
        let pending_responses = ctx
            .accounts
            .pending_responses
            .as_mut()
            .ok_or(XcallError::PendingResponsesAccountNotSpecified)?;

        if !pending_responses.sources.contains(&sender) {
            pending_responses.sources.push(sender)
        }
        if pending_responses.sources.len() != rollback.protocols().len() {
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
            // TODO:
            // close rollback account and refund lamports
            // save success reponse to an account

            if result.message().is_some() {
                let _reply_msg = result.message().unwrap();
                // TODO: handle reply
            }
        }
        _ => {
            if rollback.rollback().len() < 1 {
                return Err(XcallError::NoRollbackData.into());
            }
            rollback.enable_rollback();

            emit!(event::RollbackMessage {
                sn: result.sequence_no()
            })
        }
    }

    Ok(())
}

pub fn is_valid_source(
    default_conn: &String,
    sender: &String,
    protocols: &Vec<String>,
) -> Result<bool> {
    if protocols.contains(sender) {
        return Ok(true);
    }

    Ok(sender == default_conn)
}

#[derive(Accounts)]
#[instruction(from_nid: NetId, msg: Vec<u8>, sequence_no: u128 )]
pub struct HandleMessageCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// TODO: include hash of payload in seeds
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1024,
        seeds = [],
        bump
    )]
    pub pending_requests: Option<Account<'info, PendingRequests>>,

    /// TODO: include hash of payload in seeds
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1024,
        seeds = [],
        bump
    )]
    pub pending_responses: Option<Account<'info, PendingResponses>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1024,
        seeds = ["proxy".as_bytes(), config.last_req_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proxy_request: Account<'info, ProxyRequest>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), from_nid.to_string().as_bytes()],
        bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(mut)]
    pub rollback_accunt: Account<'info, RollbackAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct PendingRequests {
    pub sources: Vec<String>,
}

#[account]
pub struct PendingResponses {
    pub sources: Vec<String>,
}

#[account]
pub struct ProxyRequest {
    pub req: CSMessageRequest,
    pub bump: u8,
}
