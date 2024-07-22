use anchor_lang::prelude::*;
use xcall_lib::{
    message::{call_message::CallMessage, envelope::Envelope, AnyMessage},
    network_address::NetworkAddress,
    xcall_dapp_msg::HandleCallMessageResponse,
};

use crate::{state::*, xcall};

pub fn handle_call_message<'info>(
    ctx: Context<'_, '_, '_, 'info, HandleCallMessageCtx<'info>>,
    from: NetworkAddress,
    data: Vec<u8>,
    _protocols: Option<Vec<String>>,
) -> Result<HandleCallMessageResponse> {
    let (_, account) = from.parse_network_address();
    if ctx.accounts.signer.key().to_string() == account {
        return Ok(HandleCallMessageResponse {
            success: true,
            message: "success".to_owned(),
        });
    };

    let msg_data: String = rlp::decode(&data).unwrap();
    if &msg_data == "rollback" {
        return Ok(HandleCallMessageResponse {
            success: false,
            message: "Revert from dapp".to_owned(),
        });
    } else {
        if &msg_data == "reply-response" {
            let message = AnyMessage::CallMessage(CallMessage {
                data: vec![1, 2, 3],
            });

            let envelope = Envelope::new(message, Vec::new(), Vec::new());
            let msg = rlp::encode(&envelope).to_vec();

            let ix_data = xcall::get_send_call_ix_data(msg, from)?;

            xcall::call_xcall_send_call(
                &ix_data,
                &ctx.accounts.config,
                &ctx.accounts.signer,
                &ctx.accounts.system_program,
                &ctx.remaining_accounts,
            )?;
        }
    }

    return Ok(HandleCallMessageResponse {
        success: true,
        message: "success".to_owned(),
    });
}

#[derive(Accounts)]
pub struct HandleCallMessageCtx<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}
