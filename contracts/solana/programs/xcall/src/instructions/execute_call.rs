use std::{str::FromStr, vec};

use anchor_lang::prelude::*;
use xcall_lib::message::msg_type::MessageType;

use crate::{
    connection, dapp,
    error::XcallError,
    helper,
    state::*,
    types::{message::CSMessage, result::CSMessageResult},
};

/// Executes a call based on a proxy request.
///
/// This function processes a call by verifying the provided data against the request's data hash
/// and then invoking the `handle_call_message` instruction on the DApp. Depending on the message
/// type, it handles the response accordingly, potentially sending a result back through the
/// connection program.
///
/// # Parameters
/// - `ctx`: The context containing all the necessary accounts and program state.
/// - `req_id`: The unique identifier for the request being processed.
/// - `data`: The data associated with the call request, which will be verified and processed.
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the call was executed successfully, or an error if it failed.
pub fn execute_call<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteCallCtx<'info>>,
    req_id: u128,
    data: Vec<u8>,
) -> Result<()> {
    let req = &ctx.accounts.proxy_request.req;

    if helper::hash_data(&data) != req.data() {
        return Err(XcallError::DataMismatch.into());
    }

    let dapp_key = Pubkey::from_str(&req.to()).map_err(|_| XcallError::InvalidPubkey)?;

    // Prepare the instruction data for the DApp's `handle_call_message` instruction.
    let dapp_ix_data = dapp::get_handle_call_message_ix_data(
        req.from().to_owned(),
        data.clone(),
        req.protocols(),
    )?;

    // Invoke the DApp's `handle_call_message` instruction.
    let dapp_res = dapp::invoke_handle_call_message_ix(
        dapp_key,
        dapp_ix_data,
        &ctx.accounts.config,
        &ctx.accounts.signer,
        &ctx.remaining_accounts[(req.protocols().len() * 3)..],
    )?;

    // Handle the response based on the message type.
    match req.msg_type() {
        MessageType::CallMessage => {
            dapp::handle_response(req_id, dapp_res)?;
        }
        MessageType::CallMessagePersisted => {
            if !dapp_res.success {
                msg!("Error executing call from dapp: {:?}", dapp_res.message);
                return Err(XcallError::RevertFromDapp.into());
            }
            dapp::handle_response(req_id, dapp_res)?;
        }
        MessageType::CallMessageWithRollback => {
            let res_code = dapp::handle_response(req_id, dapp_res)?;

            let result = CSMessageResult::new(req.sequence_no(), res_code, None);
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
        }
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(req_id : u128, from_nid: String, conn_sn: u128, connection: Pubkey)]
pub struct ExecuteCallCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// The configuration account, which stores important settings and counters for the program.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::OnlyAdmin
    )]
    pub admin: AccountInfo<'info>,

    /// The proxy request account, identified by a request ID, which is used for executing
    /// calls. The account is closed after use, with any remaining funds sent to the `admin`.
    #[account(
        mut,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), from_nid.as_bytes(), &conn_sn.to_be_bytes(), &connection.to_bytes()],
        bump = proxy_request.bump,
        close = admin
    )]
    pub proxy_request: Account<'info, ProxyRequest>,
}
