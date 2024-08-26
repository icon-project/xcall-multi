use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction, sysvar},
};
use xcall_lib::{
    message::{envelope::Envelope, msg_trait::IMessage, AnyMessage},
    network_address::NetworkAddress,
};

use crate::{
    connection,
    error::XcallError,
    event, helper,
    state::*,
    types::{message::CSMessage, request::CSMessageRequest, rollback::Rollback},
};

/// Sends a cross-chain message to a specified network address.
///
/// This function handles encoding, validation, and sending of a cross-chain message.
/// It also manages the creation of a rollback account if needed and emits an event upon successful
/// sending
///
/// # Arguments
/// - `ctx`: The context of the solana program instruction
/// - `message`: The `Envelope` payload, encoded as rlp bytes
/// - `to`: The target network address where the message is to be sent
///
/// # Returns
/// - `Result<u128>`: The sequence number of the message if successful, wrapped in a `Result`.
pub fn send_call<'info>(
    ctx: Context<'_, '_, '_, 'info, SendCallCtx<'info>>,
    message: Vec<u8>,
    to: NetworkAddress,
) -> Result<u128> {
    let envelope: Envelope = rlp::decode(&message).map_err(|_| XcallError::DecodeFailed)?;

    let sequence_no = ctx.accounts.config.get_next_sn();
    let config = &ctx.accounts.config;

    // Determine the sender's key
    let from_key = if helper::is_program(&ctx.accounts.instruction_sysvar)? {
        let dapp_authority = ctx
            .accounts
            .dapp_authority
            .as_ref()
            .ok_or(XcallError::DappAuthorityNotProvided)?;

        helper::ensure_dapp_authority(dapp_authority.owner, dapp_authority.key())?;
        dapp_authority.owner.to_owned()
    } else {
        ctx.accounts.signer.key()
    };

    // Validate the message payload and rollback account
    validate_payload(
        &ctx.accounts.rollback_account,
        &ctx.accounts.instruction_sysvar,
        &envelope,
    )?;

    // Handle rollback logic if rollback message is present
    if envelope.message.rollback().is_some() {
        let rollback = Rollback::new(
            from_key,
            to.clone(),
            envelope.sources.clone(),
            envelope.message.rollback().unwrap(),
            false,
        );

        let rollback_account = ctx.accounts.rollback_account.as_deref_mut().unwrap();
        rollback_account.set(rollback, ctx.bumps.rollback_account.unwrap());
    }

    let from = NetworkAddress::new(&config.network_id, &from_key.to_string());

    let request = CSMessageRequest::new(
        from,
        to.account(),
        sequence_no,
        envelope.message.msg_type(),
        envelope.message.data(),
        envelope.destinations,
    );

    // Determine if a response is needed for the request
    let need_response = request.need_response();

    let cs_message = CSMessage::from(request.clone()).as_bytes();
    helper::ensure_data_length(&cs_message)?;

    let sn: i64 = if need_response { sequence_no as i64 } else { 0 };
    let ix_data = connection::get_send_message_ix_data(&to.nid(), sn, cs_message)?;

    // Send the message to all specified source addresses
    for (i, _) in envelope.sources.iter().enumerate() {
        connection::call_connection_send_message(
            i,
            &ix_data,
            &envelope.sources,
            &ctx.accounts.config,
            &ctx.accounts.signer,
            &ctx.accounts.system_program,
            &ctx.remaining_accounts,
        )?;
    }

    // If a protocol fee is configured, claim it from signer to fee handler account
    if config.protocol_fee > 0 {
        claim_protocol_fee(
            &ctx.accounts.signer,
            &ctx.accounts.fee_handler,
            &ctx.accounts.system_program,
            config.protocol_fee,
        )?;
    }

    emit!(event::CallMessageSent {
        from: from_key,
        to: to.to_string(),
        sn: sequence_no,
    });

    Ok(sequence_no)
}

/// Validates the payload of an envelope message
///
/// This function checks that the sources and destinations of the envelope are specified,
/// and that the correct conditions are met for different types of messages, including
/// handling of rollback accounts when necessary.
///
/// # Arguments
/// - `rollback_account`: An optional account that holds rollback information. This should be
///   `None` if rollback is not applicable.
/// - `sysvar_account_info`: Account information for the system variable used to determine
///   if the caller is a program.
/// - `envelope`: The envelope containing the message to be validated.
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if validation passes, or an error if validation fails.
pub fn validate_payload(
    rollback_account: &Option<Account<RollbackAccount>>,
    sysvar_account_info: &AccountInfo,
    envelope: &Envelope,
) -> Result<()> {
    if envelope.sources.is_empty() {
        return Err(XcallError::SourceProtocolsNotSpecified.into());
    }
    if envelope.destinations.is_empty() {
        return Err(XcallError::DestinationProtocolsNotSpecified.into());
    }

    match &envelope.message {
        AnyMessage::CallMessage(_) => {
            if rollback_account.is_some() {
                return Err(XcallError::RollbackAccountMustNotBeSpecified.into());
            }
        }
        AnyMessage::CallMessagePersisted(_) => {
            if rollback_account.is_some() {
                return Err(XcallError::RollbackAccountMustNotBeSpecified.into());
            }
        }
        AnyMessage::CallMessageWithRollback(msg) => {
            if rollback_account.is_none() {
                return Err(XcallError::RollbackAccountNotSpecified.into());
            }
            if !helper::is_program(sysvar_account_info)? {
                return Err(XcallError::RollbackNotPossible.into());
            }
            helper::ensure_rollback_size(&msg.rollback)?;
        }
    }

    Ok(())
}

/// Claims the protocol fee by transferring the specified amount from the signer to the fee handler.
///
/// This function creates a system instruction to transfer the `protocol_fee` from the `signer`
/// account to the `fee_handler` account using the system_program.
///
/// # Arguments
/// - `signer`: The account that is paying the protocol fee. This account must have sufficient
///   funds to cover the `protocol_fee`.
/// - `fee_handler`: The account that receives the protocol fee. Typically, this is a
///   designated fee collector or treasury account.
/// - `system_program`: The system program that facilitates the transfer. This must be the
///   standard Solana system program account.
/// - `protocol_fee`: The amount of fee, in lamports, to be transferred from the `signer` to
///   the `fee_handler`.
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the fee transfer is successful, or an error if the
///   transfer fails.
pub fn claim_protocol_fee<'info>(
    signer: &AccountInfo<'info>,
    fee_handler: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    protocol_fee: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(&signer.key(), &fee_handler.key(), protocol_fee);
    invoke(
        &ix,
        &[
            signer.to_owned(),
            fee_handler.to_owned(),
            system_program.to_owned(),
        ],
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct SendCallCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    pub dapp_authority: Option<Signer<'info>>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// CHECK: The instruction sysvar account, used to verify if the current instruction is a
    /// program invocation. This account is an unchecked account because the constraints are
    /// verified within the account trait.
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar: UncheckedAccount<'info>,

    /// The configuration account, which stores important settings and counters for the
    /// program. This account is mutable because the sequence number for messages will be updated.
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: The account designated to receive protocol fees. This account is checked
    /// against the `config.fee_handler` to ensure it is valid. This is a safe unchecked account
    /// because the validity of the fee handler is verified during instruction execution
    #[account(mut, address = config.fee_handler @ XcallError::InvalidFeeHandler)]
    pub fee_handler: AccountInfo<'info>,

    /// An optional rollback account that stores information for undoing the effects of the call
    /// if needed. The account is initialized when necessary, with the `signer` paying for its
    /// creation.
    #[account(
        init,
        payer = signer,
        space = RollbackAccount::SIZE,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &(config.sequence_no + 1).to_be_bytes()],
        bump,
      )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,
}
