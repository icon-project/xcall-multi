use anchor_lang::{
    prelude::*,
    solana_program::{hash, instruction::Instruction, program::invoke_signed},
};

use crate::{
    error::*,
    event, helper, id,
    state::*,
    types::{
        message::{CSMessage, CSMessageType},
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
    },
};

/// Handles incoming cross-chain messages by processing requests or responses from the source or
/// destination chain. This instruction serves as the entry point for handling messages received
/// from other chains. It determines the type of message (either a request from a source chain or
/// a response from a destination chain), and then invokes the appropriate inner instruction of
/// the xcall program to process the message.
///
/// # Parameters
/// - `ctx`: The context containing all necessary accounts and program-specific information.
/// - `from_nid`: The network ID of the chain that sent the message.
/// - `message`: The encoded message payload received from the chain.
/// - `sequence_no`: The sequence number associated with the message, used to track message
///   ordering and responses.
/// - `conn_sn`: The sequence number of connection associated with the message, used to derive
///   unique proxy request account with the combination of other parameters
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if successful, or an appropriate error if any validation or
///   invocation fails.
pub fn handle_message<'info>(
    ctx: Context<'_, '_, '_, 'info, HandleMessageCtx<'info>>,
    from_nid: String,
    message: Vec<u8>,
    sequence_no: u128,
    conn_sn: u128,
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

            invoke_handle_request(ctx, from_nid, cs_message.payload, conn_sn)?
        }
        CSMessageType::CSMessageResult => {
            let rollback_account = ctx
                .accounts
                .rollback_account
                .as_ref()
                .ok_or(XcallError::CallRequestNotFound)?;

            let all_sources_delivered = check_sources_and_pending_response(
                &ctx.accounts.connection,
                rollback_account.rollback.protocols(),
                &mut ctx.accounts.pending_response,
                &ctx.accounts.admin,
            )?;
            if !all_sources_delivered {
                return Ok(());
            }

            invoke_handle_result(ctx, from_nid, cs_message.payload, sequence_no, conn_sn)?;
        }
    }
    Ok(())
}

/// Processes an incoming cross-chain request, validating the source and managing the request state.
///
/// This function handles a request from a source chain, verifying the sender's network ID and
/// protocols. It manages the state of any pending requests, emits a `CallMessage` event with
/// the request's details, and stores the request data in the `ProxyRequest` account for further
/// processing.
///
/// # Parameters
/// - `ctx`: Context containing all relevant accounts and program-specific information.
/// - `from_nid`: Network ID of the source chain that sent the request.
/// - `payload`: Encoded payload of the request message.
/// - `conn_sn`: The sequence number of connection associated with the message, used to derive
///   unique proxy request account with the combination of other parameters
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the request is successfully processed, or an error if
///   validation or account updates fail.
pub fn handle_request(
    ctx: Context<HandleRequestCtx>,
    from_nid: String,
    payload: &[u8],
    conn_sn: u128,
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
            // close the proxy request as it's no longer needed
            ctx.accounts
                .proxy_request
                .close(ctx.accounts.signer.to_account_info())?;

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
        data: req.data(),
        connection: source.owner.to_owned(),
        connSn: conn_sn
    });

    let proxy_request = &mut ctx.accounts.proxy_request;

    req.hash_data();
    proxy_request.set(req, ctx.bumps.proxy_request);

    Ok(())
}

/// Handles the result of a cross-chain message response, determining the next steps
/// based on the response code.
///
/// This function processes the outcome of a cross-chain operation, performing different
/// actions depending on whether the result was successful or not. If the operation was successful,
/// it finalizes the process by closing the rollback account and marking the operation as successful.
/// If the operation failed, it enables rollback and emits an event indicating a rollback action.
/// Additionally, it ensures that the appropriate accounts are present or absent based on the outcome.
///
/// # Arguments
/// - `ctx`: The context of accounts involved in the operation.
/// - `payload`: The raw result data from the cross-chain operation, which is decoded and processed.
/// - `conn_sn`: The sequence number of connection associated with the message, used to derive
///   unique proxy request account with the combination of other parameters
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the operation completes successfully, or an error if something
/// goes wrong.
pub fn handle_result(ctx: Context<HandleResultCtx>, payload: &[u8], conn_sn: u128) -> Result<()> {
    let result: CSMessageResult = payload.try_into()?;
    let proxy_request = &ctx.accounts.proxy_request;
    let rollback_account = &mut ctx.accounts.rollback_account;

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
                handle_reply(ctx, message, conn_sn)?;
            } else {
                if proxy_request.is_some() {
                    return Err(XcallError::ProxyRequestAccountMustNotBeSpecified.into());
                }
            }
        }
        _ => {
            if ctx.accounts.successful_response.is_some() {
                return Err(XcallError::SuccessfulResponseAccountMustNotBeSpecified.into());
            }
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

/// Handles the error for a specific message sequence originally sent from Solana.
/// If something goes wrong, this function enables rollback and emits relevant events
/// to revert the state for the dApp that initiated the message. It's invoked by each
/// connection involved in sending the message from Solana to another chain.
///
/// # Arguments:
/// * `ctx` - Context containing the necessary accounts to process the rollback.
/// * `sequence_no` - The sequence number of the message to be rolled back.
///
/// # Returns
/// - `Result<()>` - Returns `Ok(())` if the rollback is successfully enabled and processed,
///   or an appropriate error if the validation or rollback fails.
pub fn handle_error(ctx: Context<HandleErrorCtx>, sequence_no: u128) -> Result<()> {
    let rollback_account = &mut ctx.accounts.rollback_account;

    let all_sources_delivered = check_sources_and_pending_response(
        &ctx.accounts.connection,
        rollback_account.rollback.protocols(),
        &mut ctx.accounts.pending_response,
        &ctx.accounts.admin,
    )?;
    if !all_sources_delivered {
        return Ok(());
    }

    emit!(event::ResponseMessage {
        code: CSResponseType::CSResponseFailure.into(),
        sn: sequence_no
    });
    emit!(event::RollbackMessage { sn: sequence_no });

    rollback_account.rollback.enable_rollback();

    Ok(())
}

/// Handles reply messages from cross-chain communication after receiving a response.
/// Verifies the origin of the reply and prepares it for further processing.
/// Emits an event with details about the message and sets up the `proxy_request`
/// for the subsequent steps.
///
/// # Arguments
/// * `ctx` - The context containing relevant accounts for handling the reply.
/// * `reply` - The mutable reference to the incoming reply message to be processed.
/// * `conn_sn`: The sequence number of connection associated with the message, used to derive
///   unique proxy request account with the combination of other parameters
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the operation completes successfully, or an error if something
/// goes wrong.
pub fn handle_reply(
    ctx: Context<HandleResultCtx>,
    reply: &mut CSMessageRequest,
    conn_sn: u128,
) -> Result<()> {
    let rollback = &ctx.accounts.rollback_account.rollback;
    if rollback.to().nid() != reply.from().nid() {
        return Err(XcallError::InvalidReplyReceived.into());
    }

    let req_id = ctx.accounts.config.get_next_req_id();

    emit!(event::CallMessage {
        from: reply.from().to_string(),
        to: reply.to().clone(),
        sn: reply.sequence_no(),
        reqId: req_id,
        data: reply.data(),
        connection: ctx.accounts.connection.owner.to_owned(),
        connSn: conn_sn
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

/// Executes the inner instruction to handle a request received from the source chain.
///
/// This function is invoked after the `handle_message` instruction determines that the received
/// message is a request from the source chain. It prepares the necessary accounts and instruction
/// data, and then calls the appropriate inner instruction of the xcall program to process the
/// request. This process ensures that the request is correctly handled and forwarded as needed
/// within the cross-chain communication context.
///
/// # Parameters
/// - `ctx`: The context containing the accounts and program-specific info needed for the instruction.
/// - `from_nid`: The network ID of the chain that sent the request.
/// - `msg_payload`: The payload of the request message received from the source chain.
/// - `conn_sn`: The sequence number of connection associated with the message, used to derive
///   unique proxy request account with the combination of other parameters
///
/// # Returns
/// - `Result<()>`: Indicates whether the invocation was successful or encountered an error.
pub fn invoke_handle_request<'info>(
    ctx: Context<'_, '_, '_, 'info, HandleMessageCtx<'info>>,
    from_nid: String,
    msg_payload: Vec<u8>,
    conn_sn: u128,
) -> Result<()> {
    let mut data = vec![];
    let args = xcall_lib::xcall_type::HandleRequestArgs {
        from_nid,
        msg_payload,
        conn_sn,
    };
    args.serialize(&mut data)?;
    let ix_data = helper::get_instruction_data(xcall_lib::xcall_type::HANDLE_REQUEST_IX, data);

    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(ctx.accounts.signer.key(), true),
        AccountMeta::new_readonly(ctx.accounts.connection.key(), true),
        AccountMeta::new_readonly(ctx.accounts.config.key(), true),
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        AccountMeta::new(ctx.accounts.admin.key(), false),
    ];
    let mut account_infos: Vec<AccountInfo<'info>> = vec![
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.connection.to_account_info(),
        ctx.accounts.config.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.admin.to_account_info(),
    ];
    for account in ctx.remaining_accounts {
        if account.is_writable {
            account_metas.push(AccountMeta::new(account.key(), account.is_signer))
        } else {
            account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer))
        }
        account_infos.push(account.to_account_info());
    }

    let ix = Instruction {
        program_id: id(),
        accounts: account_metas,
        data: ix_data.clone(),
    };

    invoke_signed(
        &ix,
        &account_infos,
        &[&[Config::SEED_PREFIX.as_bytes(), &[ctx.accounts.config.bump]]],
    )?;

    Ok(())
}

/// Executes the inner instruction to handle a response received from the destination chain.
///
/// This function is invoked after the `handle_message` instruction determines that the received
/// message is a response from the destination chain. It prepares the necessary accounts and
/// instruction data, and then calls the appropriate inner instruction of the xcall program to
/// process the response. The response is associated with a specific sequence number and may involve
/// a rollback if specified.
///
/// # Parameters
/// - `ctx`: The context containing the accounts and program-specific info needed for the instruction.
/// - `from_nid`: The network ID of the chain that sent the response.
/// - `msg_payload`: The payload of the message received from the destination chain.
/// - `sequence_no`: The sequence number associated with the original request message.
/// - `conn_sn`: The sequence number of connection associated with the message, used to derive
///   unique proxy request account with the combination of other parameters
///
/// # Returns
/// - `Result<()>`: Indicates whether the invocation was successful or encountered an error.
pub fn invoke_handle_result<'info>(
    ctx: Context<'_, '_, '_, 'info, HandleMessageCtx<'info>>,
    from_nid: String,
    msg_payload: Vec<u8>,
    sequence_no: u128,
    conn_sn: u128,
) -> Result<()> {
    let mut data = vec![];
    let args = xcall_lib::xcall_type::HandleResultArgs {
        from_nid,
        msg_payload,
        sequence_no,
        conn_sn,
    };
    args.serialize(&mut data)?;
    let ix_data = helper::get_instruction_data(xcall_lib::xcall_type::HANDLE_RESULT_IX, data);

    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(ctx.accounts.signer.key(), true),
        AccountMeta::new_readonly(ctx.accounts.connection.key(), true),
        AccountMeta::new_readonly(ctx.accounts.config.key(), true),
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        AccountMeta::new(ctx.accounts.admin.key(), false),
    ];
    let mut account_infos: Vec<AccountInfo<'info>> = vec![
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.connection.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.config.to_account_info(),
        ctx.accounts.admin.to_account_info(),
    ];

    for account in ctx.remaining_accounts {
        if account.is_writable {
            account_metas.push(AccountMeta::new(account.key(), account.is_signer))
        } else {
            account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer))
        }
        account_infos.push(account.to_account_info());
    }

    // append all accounts with lamport changes to the end of your CPI instruction accounts list
    if ctx.accounts.pending_response.is_some() {
        let pending_response = ctx.accounts.pending_response.as_ref().unwrap();

        account_metas.push(AccountMeta::new_readonly(pending_response.key(), false));
        account_infos.push(pending_response.to_account_info());
    }

    let ix = Instruction {
        program_id: id(),
        accounts: account_metas,
        data: ix_data.clone(),
    };

    invoke_signed(
        &ix,
        &account_infos,
        &[&[Config::SEED_PREFIX.as_bytes(), &[ctx.accounts.config.bump]]],
    )?;

    Ok(())
}

/// Validates the source and manages the pending response for a multi-protocol message.
///
/// This function checks if the sender is a valid source for the given protocols and updates
/// the pending response account if multiple protocols are used. It ensures that the sender
/// is listed in the pending response account and closes the account when all expected sources
/// are received.
///
/// # Parameters
/// - `sender`: The `Signer` representing the sender account to validate.
/// - `protocols`: A vector of protocol names that the sender must be listed in.
/// - `pending_response`: An optional mutable reference to the `PendingResponse` account.
/// - `admin`: The admin account for closing the `PendingResponse` account.
///
/// # Returns
/// - `Result<bool>`: `Ok(true)` if all sources are valid and the pending response is closed,
/// or `Ok(false)` if the message is still pending (not all sources have responded). Returns
/// an error if validation fails.
pub fn check_sources_and_pending_response<'info>(
    sender: &Signer,
    protocols: &Vec<String>,
    pending_response: &mut Option<Account<'info, PendingResponse>>,
    admin: &AccountInfo<'info>,
) -> Result<bool> {
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
            return Ok(false);
        }
        pending_response.close(admin.to_owned())?;
    }

    Ok(true)
}

/// Checks if the given sender is a valid source for the provided protocols.
///
/// This function verifies if the sender's authority is correct and if the sender is listed
/// among the allowed protocols. It ensures the sender has the proper authorization and
/// is recognized by the system.
///
/// # Parameters
/// - `sender`: A reference to the `Signer` representing the sender account.
/// - `protocols`: A vector of protocol names that the sender must be listed in.
///
/// # Returns
/// - `Result<bool>`: `true` if the sender is authorized and listed in the protocols,
/// `false` otherwise.
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

    /// The signer account representing the connection through which the message is being processed.
    pub connection: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::InvalidAdminKey
    )]
    pub admin: AccountInfo<'info>,

    /// The configuration account, which stores important settings and counters for the program.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// An optional account that is created when sending a rollback message to a destination chain.
    /// It stores essential details related to the message, which are required to handle the response
    /// from the destination chain. The `rollback_account` is only needed if a response is expected
    /// for a specific sequence of the message that was sent from this chain.
    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,

    /// An optional account created to track whether a response has been received from each connection
    /// specified in a message. This account is only initialized if multiple connections are used for
    /// sending and receiving messages, enhancing security by avoiding reliance on a single connection.
    #[account(
        init_if_needed,
        payer = signer,
        space = PendingResponse::SIZE,
        seeds = [PendingResponse::SEED_PREFIX.as_bytes(), &hash::hash(&msg).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,
}

#[derive(Accounts)]
#[instruction(from_nid: String, msg_payload: Vec<u8>, conn_sn: u128)]
pub struct HandleRequestCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The signer account representing the connection through which the message is being processed.
    pub connection: Signer<'info>,

    /// The xcall signer account, used to verify that the provided signer is authorized
    /// by the xcall program.
    #[account(
        owner = id()
    )]
    pub xcall: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::InvalidAdminKey
    )]
    pub admin: AccountInfo<'info>,

    /// The configuration account, which stores important settings and counters for the program.
    /// This account is mutable because the request sequence may be updated during instruction
    /// processing
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// Stores details of each cross-chain message request sent from the source to the destination chain.
    #[account(
        init_if_needed,
        payer = signer,
        space = ProxyRequest::SIZE,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), from_nid.as_bytes(), &conn_sn.to_be_bytes(), &connection.owner.to_bytes()],
        bump
    )]
    pub proxy_request: Account<'info, ProxyRequest>,

    /// Tracks the receipt of requests from a multi-connection message. This account is optional and
    /// only created if multiple connections are used to send a message, ensuring the request is fully
    /// received.
    #[account(
        init_if_needed,
        payer = signer,
        space = PendingRequest::SIZE,
        seeds = [PendingRequest::SEED_PREFIX.as_bytes(), &hash::hash(&msg_payload).to_bytes()],
        bump,
    )]
    pub pending_request: Option<Account<'info, PendingRequest>>,
}

#[derive(Accounts)]
#[instruction(from_nid: String, msg_payload: Vec<u8>, sequence_no: u128, conn_sn: u128)]
pub struct HandleResultCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The signer account representing the connection through which the message is being processed.
    pub connection: Signer<'info>,

    /// The xcall signer account, used to verify that the provided signer is authorized
    /// by the xcall program.
    #[account(
        owner = id()
    )]
    pub xcall: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::InvalidAdminKey
    )]
    pub admin: AccountInfo<'info>,

    /// The configuration account, which stores important settings and counters for the program.
    /// This account is mutable because the request sequence may be updated during instruction
    /// processing
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// A rollback account created when sending a rollback message to a destination chain.
    /// It stores essential details related to the message, necessary for handling the response
    /// from the destination chain. The `rollback_account` is required only if a response is
    /// expected for a specific sequence of the message sent from this chain.
    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Account<'info, RollbackAccount>,

    /// Stores details of each cross-chain message request sent from the source to the destination
    /// chain.
    #[account(
        init_if_needed,
        payer = signer,
        space = ProxyRequest::SIZE,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), from_nid.as_bytes(), &conn_sn.to_be_bytes(), &connection.owner.to_bytes()],
        bump
    )]
    pub proxy_request: Option<Account<'info, ProxyRequest>>,

    /// Stores details of a successful response received from the destination chain. This account
    /// is optional and created only when a successful response is expected for a specific sequence
    /// number.
    #[account(
        init_if_needed,
        payer = signer,
        space = SuccessfulResponse::SIZE,
        seeds = [SuccessfulResponse::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub successful_response: Option<Account<'info, SuccessfulResponse>>,
}

#[derive(Accounts)]
#[instruction(sequence_no: u128)]
pub struct HandleErrorCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The signer account representing the connection through which the message is being processed.
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

    /// A rollback account created when initiating a rollback message to a destination chain.
    /// This account stores crucial details about the message, which are necessary for processing
    /// the response from the destination chain. In this instruction, the `rollback_account` is
    /// used to enable and execute the rollback operation within the DApp that originally sent
    /// the message.
    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Account<'info, RollbackAccount>,

    /// An optional account created to track whether a response has been received from each connection
    /// specified in a message. This account is only initialized if multiple connections are used for
    /// sending and receiving messages, enhancing security by avoiding reliance on a single connection.
    #[account(
        init_if_needed,
        payer = signer,
        space = PendingResponse::SIZE,
        seeds = [PendingResponse::SEED_PREFIX.as_bytes(), &hash::hash(&CSMessageResult::new(sequence_no, CSResponseType::CSResponseFailure, None).as_bytes()).to_bytes()],
        bump
    )]
    pub pending_response: Option<Account<'info, PendingResponse>>,
}
