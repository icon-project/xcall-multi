use anchor_lang::prelude::*;
use xcall_lib::{network_address::NetworkAddress, xcall_dapp_type::HandleCallMessageResponse};

use crate::{error::*, state::*};

use std::str;

pub fn handle_call_message<'info>(
    ctx: Context<'_, '_, '_, 'info, HandleCallMessageCtx<'info>>,
    from: NetworkAddress,
    data: Vec<u8>,
    _protocols: Vec<String>,
) -> Result<HandleCallMessageResponse> {
    let (_, account) = from.parse_network_address();
    if ctx.accounts.signer.key().to_string() == account {
        return Ok(HandleCallMessageResponse {
            success: true,
            message: "success".to_owned(),
        });
    };

    let msg_data = str::from_utf8(&data).unwrap();
    if msg_data == "rollback" {
        return Ok(HandleCallMessageResponse {
            success: false,
            message: "Revert from dapp".to_owned(),
        });
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

    #[account(
        owner = config.xcall_address @ DappError::OnlyXcall
    )]
    pub xcall: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}
