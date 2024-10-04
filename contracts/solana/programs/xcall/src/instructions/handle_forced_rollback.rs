use std::{str::FromStr, vec};

use anchor_lang::prelude::*;
use xcall_lib::message::msg_type::MessageType;

use crate::{
    connection,
    error::XcallError,
    helper,
    state::*,
    types::{
        message::CSMessage,
        result::{CSMessageResult, CSResponseType},
    },
};

/// Handles a forced rollback of a cross-chain message when an unknown error occurs after
/// the message is received on the destination chain from the source chain.
/// This allows the dApp to trigger a rollback, sending a failure response back to the source chain.
///
/// The function verifies the dApp's authority, ensures the message type supports rollback,
/// and constructs a failure response to be sent back across the specified protocols (connections).
/// The rollback is enforced by sending a failure message for each protocol involved in the
/// original message, enabling the state to revert as though the message was never successfully
/// processed.
///
/// # Arguments
/// * `ctx` - Context containing the accounts required for processing the forced rollback.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` on successful execution, or an error if any validation
///   or execution step fails.
pub fn handle_forced_rollback<'info>(
    ctx: Context<'_, '_, '_, 'info, HandleForcedRollbackCtx<'info>>,
) -> Result<()> {
    let req = &ctx.accounts.proxy_request.req;

    // Ensures that the request is not received by specified protocols
    if req.to().is_empty() {
        return Err(XcallError::RequestPending.into());
    }

    if req.msg_type() != MessageType::CallMessageWithRollback {
        return Err(XcallError::RollbackNotPossible.into());
    }

    let dapp_authority = &ctx.accounts.dapp_authority;
    helper::ensure_dapp_authority(dapp_authority.owner, dapp_authority.key())?;

    let to = Pubkey::from_str(&req.to()).map_err(|_| XcallError::InvalidPubkey)?;
    if dapp_authority.owner != &to {
        return Err(XcallError::InvalidSigner.into());
    }

    let result = CSMessageResult::new(req.sequence_no(), CSResponseType::CSResponseFailure, None);
    let cs_message = rlp::encode(&CSMessage::from(result)).to_vec();

    let ix_data = connection::get_send_message_ix_data(
        &req.from().nid(),
        -(req.sequence_no() as i64),
        cs_message,
    )?;

    for (i, _) in req.protocols().iter().enumerate() {
        connection::call_connection_send_message(
            i,
            &ix_data,
            &req.protocols(),
            &ctx.accounts.config,
            &ctx.accounts.signer,
            &ctx.accounts.system_program,
            &ctx.remaining_accounts,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(req_id: u128)]
pub struct HandleForcedRollbackCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable because
    /// it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The account representing the dApp authority, which must sign the transaction to enforce
    /// the rollback.
    pub dapp_authority: Signer<'info>,

    /// The Solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// The configuration account, which stores important settings and counters for the program.
    /// The `seeds` and `bump` ensure that this account is securely derived.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is safe because this account is checked against `config.admin` to ensure
    /// it matches the expected admin address.
    #[account(
        mut,
        address = config.admin @ XcallError::OnlyAdmin
    )]
    pub admin: AccountInfo<'info>,

    /// The proxy request account, identified by a request ID. This account is used for executing
    /// calls and is closed after use, with any remaining funds sent to the `admin`.
    #[account(
        mut,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), &req_id.to_be_bytes()],
        bump = proxy_request.bump,
        close = admin
    )]
    pub proxy_request: Account<'info, ProxyRequest>,
}
