use std::{str::FromStr, vec};

use anchor_lang::prelude::*;
use xcall_lib::message::msg_type::MessageType;

use crate::{
    connection, dapp,
    error::XcallError,
    helper,
    state::*,
    types::{
        message::CSMessage,
        result::{CSMessageResult, CSResponseType},
    },
};

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
    let protocols = if req.protocols().len() > 0 {
        Some(req.protocols())
    } else {
        None
    };

    let dapp_ix_data =
        dapp::get_handle_call_message_ix_data(req.from().nid(), data.clone(), protocols)?;

    let dapp_res = dapp::invoke_handle_call_message_ix(
        dapp_key,
        dapp_ix_data,
        &ctx.accounts.config,
        &ctx.accounts.signer,
        &ctx.accounts.system_program,
        &ctx.remaining_accounts,
    )?;

    match req.msg_type() {
        MessageType::CallMessage => {
            dapp::handle_response(req_id, dapp_res)?;
        }
        MessageType::CallMessagePersisted => {}
        MessageType::CallMessageWithRollback => {
            ctx.accounts.config.set_reply_state(Some(req.clone()));

            let res_code = dapp::handle_response(req_id, dapp_res)?;
            let config = &mut ctx.accounts.config;

            config.set_call_reply(None);
            config.set_reply_state(None);

            let mut msg = Vec::new();
            if config.call_reply.is_some() && res_code == CSResponseType::CSResponseSuccess {
                msg = rlp::encode(config.call_reply.as_mut().unwrap()).to_vec();
            }

            let result = CSMessageResult::new(req.sequence_no(), res_code, Some(msg));
            let cs_message = rlp::encode(&CSMessage::from(result)).to_vec();

            let mut protocols = req.protocols();
            if protocols.is_empty() {
                protocols = vec![ctx.accounts.default_connection.key().to_string()]
            }

            let ix_data = connection::get_send_message_ix_data(
                req.to(),
                -(req.sequence_no() as i64),
                cs_message,
            )?;

            for (i, _) in protocols.iter().enumerate() {
                connection::call_connection_send_message(
                    i,
                    &ix_data,
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
#[instruction(req_id : u128, data:Vec<u8>, from_nid: String)]
pub struct ExecuteCallCtx<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
        has_one = admin
    )]
    pub config: Account<'info, Config>,

    /// CHECK: this is safe because we are verifying if the passed account is admin or not
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), &req_id.to_be_bytes()],
        bump = proxy_request.bump,
        close = admin
    )]
    pub proxy_request: Account<'info, ProxyRequest>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), from_nid.as_bytes()],
        bump = default_connection.bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,
}
