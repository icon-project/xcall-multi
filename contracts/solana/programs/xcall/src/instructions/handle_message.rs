use anchor_lang::{prelude::*, solana_program::hash};

use crate::{
    error::*,
    event, helper,
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
        CSMessageType::CSMessageRequest => {
            if ctx.accounts.pending_response.is_some() {
                return Err(XcallError::PendingResponseAccountMustNotBeSpecified.into());
            }
            if ctx.accounts.successful_response.is_some() {
                return Err(XcallError::SuccessfulResponseAccountMustNotBeSpecified.into());
            }

            handle_request(ctx, from_nid, cs_message.payload())
        }
        CSMessageType::CSMessageResult => {
            if ctx.accounts.pending_request.is_some() {
                return Err(XcallError::PendingRequestAccountMustNotBeSpecified.into());
            }

            handle_result(ctx, cs_message.payload())
        }
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
    let source = &ctx.accounts.connection;
    let source_valid = is_valid_source(&source, &req.protocols())?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    }

    if req.protocols().len() > 1 {
        let pending_request = ctx
            .accounts
            .pending_request
            .as_mut()
            .ok_or(XcallError::PendingRequestAccountNotSpecified)?;

        if !pending_request.sources.contains(&source.owner) {
            pending_request.sources.push(source.owner.to_owned())
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

    let proxy_request = ctx.accounts.proxy_request.as_deref_mut().unwrap();

    req.hash_data();
    proxy_request.set(req, ctx.bumps.proxy_request.unwrap());

    Ok(())
}

pub fn handle_result(ctx: Context<HandleMessageCtx>, payload: &[u8]) -> Result<()> {
    let result: CSMessageResult = payload.try_into()?;
    let proxy_request = &ctx.accounts.proxy_request;

    let rollback_account = ctx
        .accounts
        .rollback_account
        .as_mut()
        .ok_or(XcallError::CallRequestNotFound)?;

    validate_source_and_pending_response(
        &ctx.accounts.connection,
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

            if let Some(message) = &mut result.message() {
                handle_reply(ctx, message)?;
            } else {
                if proxy_request.is_some() {
                    return Err(XcallError::ProxyRequestAccountMustNotBeSpecified.into());
                }
            }
        }
        _ => {
            if proxy_request.is_some() {
                return Err(XcallError::ProxyRequestAccountMustNotBeSpecified.into());
            }

            rollback_account.rollback.enable_rollback();

            emit!(event::RollbackMessage {
                sn: result.sequence_no()
            });
        }
    }

    Ok(())
}

pub fn handle_error(ctx: Context<HandleErrorCtx>, sequence_no: u128) -> Result<()> {
    let rollback_account = &mut ctx.accounts.rollback_account;

    validate_source_and_pending_response(
        &ctx.accounts.connection,
        rollback_account.rollback.protocols(),
        &mut ctx.accounts.pending_response,
        &ctx.accounts.admin,
    )?;

    emit!(event::ResponseMessage {
        code: CSResponseType::CSResponseFailure.into(),
        sn: sequence_no
    });
    emit!(event::RollbackMessage { sn: sequence_no });

    rollback_account.rollback.enable_rollback();

    Ok(())
}

pub fn handle_reply(ctx: Context<HandleMessageCtx>, reply: &mut CSMessageRequest) -> Result<()> {
    let rollback = &ctx.accounts.rollback_account.as_deref().unwrap().rollback;
    if rollback.to().nid() != reply.from().nid() {
        return Err(XcallError::InvalidReplyReceived.into());
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
    reply.set_protocols(rollback.protocols().clone());
    proxy_request.set(reply.to_owned(), ctx.bumps.proxy_request.unwrap());

    Ok(())
}

pub fn validate_source_and_pending_response<'info>(
    sender: &Signer,
    protocols: &Vec<String>,
    pending_response: &mut Option<Account<'info, PendingResponse>>,
    admin: &AccountInfo<'info>,
) -> Result<()> {
    let source_valid = is_valid_source(&sender, protocols)?;
    if !source_valid {
        return Err(XcallError::ProtocolMismatch.into());
    };

    if protocols.len() > 1 {
        let pending_response = pending_response
            .as_mut()
            .ok_or(XcallError::PendingResponseAccountNotSpecified)?;

        if !pending_response.sources.contains(&sender.owner) {
            pending_response.sources.push(sender.owner.to_owned())
        }
        if pending_response.sources.len() != protocols.len() {
            return Ok(());
        }
        pending_response.close(admin.to_owned())?;
    }

    Ok(())
}

pub fn is_valid_source(sender: &Signer, protocols: &Vec<String>) -> Result<bool> {
    helper::ensure_connection_authority(sender.owner, sender.key())?;
    if protocols.contains(&sender.owner.to_string()) {
        return Ok(true);
    }

    Ok(false)
}

#[derive(Accounts)]
#[instruction(from_nid: String, msg: Vec<u8>, sequence_no: u128)]
pub struct HandleMessageCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    pub connection: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// The configuration account, which stores important settings and counters for the
    /// program. This account is mutable because the last request ID of config will be updated.
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Box<Account<'info, Config>>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::InvalidAdminKey
    )]
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
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    pub connection: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// The configuration account, which stores important settings and counters for the
    /// program. This account is mutable because the last request ID of config will be updated.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::InvalidAdminKey
    )]
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
