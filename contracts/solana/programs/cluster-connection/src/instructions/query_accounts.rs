use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke},
        system_program,
    },
};
use xcall_lib::{
    query_account_type::{AccountMetadata, QueryAccountsPaginateResponse, QueryAccountsResponse},
    xcall_connection_type,
    xcall_type::{self, QUERY_HANDLE_ERROR_ACCOUNTS_IX, QUERY_HANDLE_MESSAGE_ACCOUNTS_IX},
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
        &[
            Receipt::SEED_PREFIX.as_bytes(),
            src_network.as_bytes(),
            &conn_sn.to_be_bytes(),
        ],
        &id(),
    );
    let (authority, _) = Pubkey::find_program_address(
        &[xcall_connection_type::CONNECTION_AUTHORITY_SEED.as_bytes()],
        &id(),
    );

    let mut account_metas = vec![
        AccountMetadata::new(system_program::id(), false),
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(receipt, false),
        AccountMetadata::new(authority, false),
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

    let ix_data = get_handle_message_ix_data(src_network, msg, sequence_no)?;

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

    Ok(QueryAccountsPaginateResponse::new(
        account_metas,
        page,
        limit,
    ))
}

pub fn query_revert_message_accounts(
    ctx: Context<QueryAccountsCtx>,
    sequence_no: u128,
    page: u8,
    limit: u8,
) -> Result<QueryAccountsPaginateResponse> {
    let config = &ctx.accounts.config;
    let (authority, _) = Pubkey::find_program_address(
        &[xcall_connection_type::CONNECTION_AUTHORITY_SEED.as_bytes()],
        &id(),
    );

    let mut account_metas = vec![
        AccountMetadata::new(system_program::id(), false),
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(authority, false),
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

    let ix_data = get_handle_error_ix_data(sequence_no)?;

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
    account_metas.push(AccountMetadata::new(config.xcall, false));

    Ok(QueryAccountsPaginateResponse::new(
        account_metas,
        page,
        limit,
    ))
}

pub fn get_handle_error_ix_data(sequence_no: u128) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = xcall_type::HandleErrorArgs { sequence_no };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(QUERY_HANDLE_ERROR_ACCOUNTS_IX, ix_args_data);
    Ok(ix_data)
}

pub fn get_handle_message_ix_data(
    from_nid: String,
    message: Vec<u8>,
    sequence_no: u128,
) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = xcall_type::HandleMessageArgs {
        from_nid,
        message,
        sequence_no,
    };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(QUERY_HANDLE_MESSAGE_ACCOUNTS_IX, ix_args_data);
    Ok(ix_data)
}

#[derive(Accounts)]
pub struct QueryAccountsCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
}
