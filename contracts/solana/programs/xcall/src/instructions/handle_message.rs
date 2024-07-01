use anchor_lang::{prelude::*, solana_program::hash};

use crate::{
    error::*,
    event,
    state::*,
    types::{
        message::{CSMessage, CSMessageType},
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
        rollback::Rollback,
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
    let source_valid = is_valid_source(
        &ctx.accounts.default_connection.key(),
        &source.to_string(),
        &req.protocols(),
    )?;
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
        if let Some(creator) = ctx.accounts.pending_request_creator.to_owned() {
            require_eq!(creator.key(), *pending_request.sources.get(0).unwrap());
            pending_request.close(creator)?;
        } else {
            return Err(XcallError::PendingRequestCreatorNotSpecified.into());
        }
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
        .set(req, ctx.accounts.signer.key(), ctx.bumps.proxy_request);

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

    let rollback = &mut rollback_account.rollback;

    validate_source_and_pending_response(
        sender,
        rollback.protocols(),
        ctx.accounts.default_connection.key(),
        &mut ctx.accounts.pending_response,
        &ctx.accounts.pending_response_creator,
    )?;

    let response_code = result.response_code();

    emit!(event::ResponseMessage {
        code: response_code.clone().into(),
        sn: result.sequence_no()
    });

    match response_code {
        CSResponseType::CSResponseSuccess => {
            if let Some(creator) = ctx.accounts.rollback_creator.to_owned() {
                require_eq!(creator.key(), rollback_account.owner);
                rollback_account.close(creator)?;
            } else {
                return Err(XcallError::RollbackCreatorNotSpecified.into());
            }

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
        _ => handle_rollback(rollback, result.sequence_no())?,
    }

    Ok(())
}

pub fn handle_error(ctx: Context<HandleErrorCtx>, sequence_no: u128) -> Result<()> {
    let sender = ctx.accounts.connection.owner.key();
    let rollback = &mut ctx.accounts.rollback_account.rollback;

    validate_source_and_pending_response(
        sender,
        rollback.protocols(),
        ctx.accounts.default_connection.key(),
        &mut ctx.accounts.pending_response,
        &ctx.accounts.pending_response_creator.clone(),
    )?;

    emit!(event::ResponseMessage {
        code: CSResponseType::CSResponseFailure.into(),
        sn: sequence_no
    });

    handle_rollback(rollback, sequence_no)
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
    ctx.accounts.proxy_request.set(
        reply.to_owned(),
        ctx.accounts.signer.key(),
        ctx.bumps.proxy_request,
    );

    Ok(())
}

pub fn validate_source_and_pending_response<'info>(
    sender: Pubkey,
    protocols: &Vec<String>,
    default_connection: Pubkey,
    pending_response: &mut Option<Account<'info, PendingResponse>>,
    pending_response_creator: &Option<AccountInfo<'info>>,
) -> Result<()> {
    let source_valid = is_valid_source(&default_connection.key(), &sender.to_string(), protocols)?;
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
        if let Some(creator) = pending_response_creator {
            require_eq!(creator.key(), *pending_response.sources.get(0).unwrap());
            pending_response.close(creator.to_owned())?;
        } else {
            return Err(XcallError::PendingResponseCreatorNotSpecified.into());
        }
    }

    Ok(())
}

pub fn handle_rollback(rollback: &mut Rollback, sequence_no: u128) -> Result<()> {
    if rollback.rollback().len() < 1 {
        return Err(XcallError::NoRollbackData.into());
    }
    rollback.enable_rollback();

    emit!(event::RollbackMessage { sn: sequence_no });

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
#[instruction(from_nid: String, msg: Vec<u8>, sequence_no: u128)]
pub struct HandleMessageCtx<'info> {
    pub connection: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

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
        init_if_needed,
        payer = signer,
        space = PendingRequest::SIZE,
        seeds = [PendingRequest::SEED_PREFIX.as_bytes(), &hash::hash(&msg).to_bytes()],
        bump
    )]
    pub pending_request: Option<Account<'info, PendingRequest>>,

    #[account(mut)]
    pub pending_request_creator: Option<AccountInfo<'info>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = PendingResponse::SIZE,
        seeds = [PendingResponse::SEED_PREFIX.as_bytes(), &hash::hash(&msg).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,

    #[account(mut)]
    pub pending_response_creator: Option<AccountInfo<'info>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = SuccessfulResponse::SIZE,
        seeds = [SuccessfulResponse::SEED_PREFIX.as_bytes(), sequence_no.to_string().as_bytes()],
        bump
    )]
    pub successful_response: Option<Account<'info, SuccessfulResponse>>,

    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), sequence_no.to_string().as_bytes()],
        bump
    )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,

    #[account(mut)]
    pub rollback_creator: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
#[instruction(from_nid: String, sequence_no: u128)]
pub struct HandleErrorCtx<'info> {
    pub connection: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), from_nid.as_bytes()],
        bump = default_connection.bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(
        init_if_needed,
        payer = signer,
        space = PendingResponse::SIZE,
        seeds = [PendingResponse::SEED_PREFIX.as_bytes(), &hash::hash(&CSMessageResult::new(sequence_no, CSResponseType::CSResponseFailure, None).as_bytes()).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,

    #[account(mut)]
    pub pending_response_creator: Option<AccountInfo<'info>>,

    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), sequence_no.to_string().as_bytes()],
        bump
    )]
    pub rollback_account: Account<'info, RollbackAccount>,
}
