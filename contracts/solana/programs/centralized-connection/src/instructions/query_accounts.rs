use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke},
        system_program,
    },
};
use xcall_lib::query_account_types::{
    AccountMetadata, QueryAccountsPaginateResponse, QueryAccountsResponse,
};

use crate::{helper, id, state::*};

pub fn query_send_message_accounts<'info>(
    ctx: Context<'_, '_, '_, 'info, QueryAccountsCtx<'info>>,
    dst_network: String,
) -> Result<QueryAccountsResponse> {
    let config = &ctx.accounts.config;

    let (network_fee, _) = Pubkey::find_program_address(
        &[NetworkFee::SEED_PREFIX.as_bytes(), dst_network.as_bytes()],
        &id(),
    );

    let account_metas = vec![
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(network_fee, false),
    ];

    Ok(QueryAccountsResponse {
        accounts: account_metas,
    })
}

pub fn query_recv_message_accounts(
    ctx: Context<QueryAccountsCtx>,
    src_network: String,
    conn_sn: u128,
    msg: Vec<u8>,
    sequence_no: u128,
    page: u8,
    limit: u8,
) -> Result<QueryAccountsPaginateResponse> {
    let config = &ctx.accounts.config;
    let (receipt, _) = Pubkey::find_program_address(
        &[Receipt::SEED_PREFIX.as_bytes(), &conn_sn.to_be_bytes()],
        &id(),
    );

    let mut account_metas = vec![
        AccountMetadata::new(system_program::id(), false),
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(receipt, false),
    ];

    let mut xcall_account_metas = vec![];
    let mut xcall_account_infos = vec![];

    for (_, account) in ctx.remaining_accounts.iter().enumerate() {
        if account.is_writable {
            xcall_account_metas.push(AccountMeta::new(account.key(), account.is_signer));
        } else {
            xcall_account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer));
        }

        xcall_account_infos.push(account.to_account_info())
    }

    let mut data = vec![];
    let args = QueryHandleMessage {
        from_nid: src_network,
        msg,
        sequence_no,
    };
    args.serialize(&mut data)?;

    let ix_data = helper::get_instruction_data("query_handle_message_accounts", data);

    let ix = Instruction {
        program_id: config.xcall,
        accounts: xcall_account_metas,
        data: ix_data,
    };

    invoke(&ix, &xcall_account_infos)?;

    let (_, data) = get_return_data().unwrap();
    let mut data_slice: &[u8] = &data;
    let res = QueryAccountsResponse::deserialize(&mut data_slice)?;
    let mut res_accounts = res.accounts;

    account_metas.append(&mut res_accounts);

    let offset = ((page - 1) * limit) as usize;
    let total = account_metas.len();
    let max: usize = if offset + limit as usize > total {
        total
    } else {
        offset + limit as usize
    };

    Ok(QueryAccountsPaginateResponse {
        accounts: account_metas[offset..max].to_vec(),
        total_accounts: total as u8,
        limit,
        page,
        has_next_page: total > max,
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

#[derive(Debug, Default, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct QueryHandleMessage {
    pub from_nid: String,
    pub msg: Vec<u8>,
    pub sequence_no: u128,
}
