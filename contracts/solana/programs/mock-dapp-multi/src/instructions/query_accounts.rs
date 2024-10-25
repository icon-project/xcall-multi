use anchor_lang::{prelude::*, solana_program::system_program};
use xcall_lib::query_account_type::{AccountMetadata, QueryAccountsResponse};

use crate::state::*;

pub fn query_handle_call_message_accounts(
    ctx: Context<QueryAccountsCtx>,
) -> Result<QueryAccountsResponse> {
    let config = &ctx.accounts.config;

    let account_metas = vec![
        AccountMetadata::new_readonly(system_program::id(), false),
        AccountMetadata::new(config.key(), false),
    ];

    Ok(QueryAccountsResponse {
        accounts: account_metas,
    })
}

#[derive(Accounts)]
pub struct QueryAccountsCtx<'info> {
    #[account(
      seeds = [Config::SEED_PREFIX.as_bytes()],
      bump = config.bump,
  )]
    pub config: Account<'info, Config>,
}
