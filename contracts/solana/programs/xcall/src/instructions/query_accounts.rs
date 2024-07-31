use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{
        hash,
        instruction::Instruction,
        program::{get_return_data, invoke},
        system_program,
    },
};
use xcall_lib::{
    network_address::NetworkAddress,
    query_account_type::{AccountMetadata, QueryAccountsPaginateResponse, QueryAccountsResponse},
    xcall_connection_type::QUERY_SEND_MESSAGE_ACCOUNTS_IX,
    xcall_dapp_type::QUERY_HANDLE_CALL_MESSAGE_IX,
};

use crate::{
    error::*,
    helper, id,
    state::*,
    types::{
        message::{CSMessage, CSMessageType},
        request::CSMessageRequest,
        result::{CSMessageResult, CSResponseType},
    },
};

pub fn query_handle_message_accounts(
    ctx: Context<QueryHandleMessageAccountsCtx>,
    msg: Vec<u8>,
) -> Result<QueryAccountsResponse> {
    let config = &ctx.accounts.config;
    let admin = config.admin;

    let (proxy_request, _) = Pubkey::find_program_address(
        &[
            ProxyRequest::SEED_PREFIX.as_bytes(),
            &(config.last_req_id + 1).to_be_bytes(),
        ],
        &id(),
    );

    let mut account_metas = vec![
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(admin, false),
    ];

    let cs_message: CSMessage = msg.clone().try_into()?;
    match cs_message.message_type() {
        CSMessageType::CSMessageRequest => {
            let request: CSMessageRequest = cs_message.payload().try_into()?;

            let (pending_request, _) = Pubkey::find_program_address(
                &[
                    PendingRequest::SEED_PREFIX.as_bytes(),
                    &hash::hash(&msg).to_bytes(),
                ],
                &id(),
            );

            account_metas.push(AccountMetadata::new(proxy_request, false));

            if request.protocols().len() > 1 {
                account_metas.push(AccountMetadata::new(pending_request, false))
            } else {
                account_metas.push(AccountMetadata::new(id(), false))
            }

            account_metas.push(AccountMetadata::new(id(), false));
            account_metas.push(AccountMetadata::new(id(), false));
            account_metas.push(AccountMetadata::new(id(), false));
        }
        CSMessageType::CSMessageResult => {
            let result: CSMessageResult = cs_message.payload().try_into()?;

            let sequence_no = result.sequence_no();

            let (pending_response, _) = Pubkey::find_program_address(
                &[
                    PendingResponse::SEED_PREFIX.as_bytes(),
                    &hash::hash(&msg).to_bytes(),
                ],
                &id(),
            );
            let (successful_response, _) = Pubkey::find_program_address(
                &[
                    SuccessfulResponse::SEED_PREFIX.as_bytes(),
                    &sequence_no.to_be_bytes(),
                ],
                &id(),
            );

            if result.response_code() == &CSResponseType::CSResponseSuccess
                && result.message().is_some()
            {
                account_metas.push(AccountMetadata::new(proxy_request, false))
            } else {
                account_metas.push(AccountMetadata::new(id(), false))
            }

            account_metas.push(AccountMetadata::new(id(), false));

            let rollback_account = ctx
                .accounts
                .rollback_account
                .as_ref()
                .ok_or(XcallError::RollbackAccountNotSpecified)?;

            if rollback_account.rollback.protocols().len() > 1 {
                account_metas.push(AccountMetadata::new(pending_response, false));
            } else {
                account_metas.push(AccountMetadata::new(id(), false))
            }

            if result.response_code() == &CSResponseType::CSResponseSuccess {
                account_metas.push(AccountMetadata::new(successful_response, false))
            } else {
                account_metas.push(AccountMetadata::new(id(), false));
            }

            let (rollback_account, _) = Pubkey::find_program_address(
                &[
                    RollbackAccount::SEED_PREFIX.as_bytes(),
                    &sequence_no.to_be_bytes(),
                ],
                &id(),
            );
            account_metas.push(AccountMetadata::new(rollback_account, false));
        }
    }

    Ok(QueryAccountsResponse {
        accounts: account_metas,
    })
}

pub fn query_execute_call_accounts(
    ctx: Context<QueryExecuteCallAccountsCtx>,
    req_id: u128,
    data: Vec<u8>,
    page: u8,
    limit: u8,
) -> Result<QueryAccountsPaginateResponse> {
    let config = &ctx.accounts.config;
    let (proxy_request, _) = Pubkey::find_program_address(
        &[ProxyRequest::SEED_PREFIX.as_bytes(), &req_id.to_be_bytes()],
        &id(),
    );

    let mut account_metadata = vec![
        AccountMetadata::new_readonly(system_program::id(), false),
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(config.admin, false),
        AccountMetadata::new(proxy_request, false),
    ];

    let protocols = ctx.accounts.proxy_request.req.protocols();

    let conn_ix_data =
        get_query_send_message_accounts_ix_data(ctx.accounts.proxy_request.req.from().nid())?;

    for (i, source) in protocols.iter().enumerate() {
        let connection_key = Pubkey::from_str(&source).map_err(|_| XcallError::InvalidPubkey)?;

        let res = query_connection_send_message_accoounts(
            i,
            connection_key,
            conn_ix_data.clone(),
            ctx.remaining_accounts,
        )?;

        let mut res_accounts = res.accounts;
        account_metadata.push(AccountMetadata::new(connection_key, false));
        account_metadata.append(&mut res_accounts);
    }

    let proxy_req = &ctx.accounts.proxy_request.req;
    let sources = if proxy_req.protocols().is_empty() {
        None
    } else {
        Some(proxy_req.protocols())
    };

    let dapp_ix_data =
        get_query_handle_call_message_ix_data(proxy_req.from().to_owned(), data, sources)?;

    let dapp_key = Pubkey::from_str(proxy_req.to()).map_err(|_| XcallError::InvalidPubkey)?;

    let res = query_dapp_handle_call_message_accounts(
        dapp_key,
        dapp_ix_data,
        &ctx.remaining_accounts[(protocols.len())..],
    )?;

    let mut res_accounts = res.accounts;
    account_metadata.append(&mut res_accounts);
    account_metadata.push(AccountMetadata::new_readonly(dapp_key, false));

    Ok(QueryAccountsPaginateResponse::new(
        account_metadata,
        page,
        limit,
    ))
}

pub fn query_execute_rollback_accounts(
    ctx: Context<QueryExecuteRollbackAccountsCtx>,
    page: u8,
    limit: u8,
) -> Result<QueryAccountsPaginateResponse> {
    let config = &ctx.accounts.config;
    let rollback_account = &ctx.accounts.rollback_account;
    let rollback = &rollback_account.rollback;

    let mut account_metas = vec![
        AccountMetadata::new_readonly(system_program::id(), false),
        AccountMetadata::new_readonly(config.key(), false),
        AccountMetadata::new(config.admin, false),
        AccountMetadata::new(rollback_account.key(), false),
    ];

    let protocols = if rollback.protocols().len() > 0 {
        Some(rollback.protocols().to_owned())
    } else {
        None
    };

    let ix_data = get_query_handle_call_message_ix_data(
        NetworkAddress::new(&config.network_id, &id().to_string()),
        rollback.rollback().to_owned(),
        protocols,
    )?;

    let dapp_key = rollback.from().to_owned();

    let res = query_dapp_handle_call_message_accounts(dapp_key, ix_data, &ctx.remaining_accounts)?;

    let mut res_accounts = res.accounts;
    account_metas.append(&mut res_accounts);
    account_metas.push(AccountMetadata::new_readonly(dapp_key, false));

    Ok(QueryAccountsPaginateResponse::new(
        account_metas,
        page,
        limit,
    ))
}

pub fn query_handle_error_accounts(
    ctx: Context<QueryHandleErrorAccountsCtx>,
    sequence_no: u128,
) -> Result<QueryAccountsResponse> {
    let config = &ctx.accounts.config;
    let rollback_account = &ctx.accounts.rollback_account;

    let mut account_metas = vec![
        AccountMetadata::new(config.key(), false),
        AccountMetadata::new(config.admin, false),
        AccountMetadata::new(rollback_account.key(), false),
    ];

    if rollback_account.rollback.protocols().len() > 1 {
        let msg = CSMessageResult::new(sequence_no, CSResponseType::CSResponseFailure, None);
        let (pending_response, _) = Pubkey::find_program_address(
            &[
                PendingResponse::SEED_PREFIX.as_bytes(),
                &hash::hash(&msg.as_bytes()).to_bytes(),
            ],
            &id(),
        );
        account_metas.push(AccountMetadata::new(pending_response, false));
    } else {
        account_metas.push(AccountMetadata::new(id(), false))
    }

    Ok(QueryAccountsResponse {
        accounts: account_metas,
    })
}

pub fn query_dapp_handle_call_message_accounts<'info>(
    dapp_key: Pubkey,
    ix_data: Vec<u8>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<QueryAccountsResponse> {
    let mut account_metas = vec![];
    let mut account_infos = vec![];

    for (_, account) in remaining_accounts.iter().enumerate() {
        if account.is_writable {
            account_metas.push(AccountMeta::new(account.key(), account.is_signer));
        } else {
            account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer));
        }
        account_infos.push(account.to_account_info())
    }

    let ix = Instruction {
        program_id: dapp_key,
        accounts: account_metas,
        data: ix_data,
    };

    invoke(&ix, &account_infos)?;

    let (_, data) = get_return_data().unwrap();
    let mut data_slice: &[u8] = &data;
    let res = QueryAccountsResponse::deserialize(&mut data_slice)?;

    Ok(res)
}

pub fn query_connection_send_message_accoounts<'info>(
    i: usize,
    connection_key: Pubkey,
    ix_data: Vec<u8>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<QueryAccountsResponse> {
    let conn_config = &remaining_accounts[i];

    let account_metas = vec![AccountMeta::new(conn_config.key(), false)];
    let account_infos = vec![conn_config.to_account_info()];

    let ix = Instruction {
        program_id: connection_key,
        accounts: account_metas,
        data: ix_data,
    };

    invoke(&ix, &account_infos)?;

    let (_, data) = get_return_data().unwrap();
    let mut data_slice: &[u8] = &data;
    let res = QueryAccountsResponse::deserialize(&mut data_slice)?;

    Ok(res)
}

pub fn get_query_send_message_accounts_ix_data(dst_network: String) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = QuerySendMessage { to: dst_network };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(QUERY_SEND_MESSAGE_ACCOUNTS_IX, ix_args_data);
    Ok(ix_data)
}

pub fn get_query_handle_call_message_ix_data(
    from: NetworkAddress,
    data: Vec<u8>,
    protocols: Option<Vec<String>>,
) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = QueryHandleCallMessage {
        from,
        data,
        protocols,
    };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(QUERY_HANDLE_CALL_MESSAGE_IX, ix_args_data);
    Ok(ix_data)
}

#[derive(Accounts)]
#[instruction(req_id: u128, data: Vec<u8>)]
pub struct QueryExecuteCallAccountsCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), &req_id.to_be_bytes()],
        bump = proxy_request.bump
    )]
    pub proxy_request: Account<'info, ProxyRequest>,
}

#[derive(Accounts)]
#[instruction(from_nid: String, msg: Vec<u8>, sequence_no: u128)]
pub struct QueryHandleMessageAccountsCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,
}

#[derive(Accounts)]
#[instruction(sn: u128)]
pub struct QueryExecuteRollbackAccountsCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sn.to_be_bytes()],
        bump = rollback_account.bump
    )]
    pub rollback_account: Account<'info, RollbackAccount>,
}

#[derive(Accounts)]
#[instruction(sequence_no: u128)]
pub struct QueryHandleErrorAccountsCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump
    )]
    pub rollback_account: Account<'info, RollbackAccount>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct QuerySendMessage {
    to: String,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct QueryHandleCallMessage {
    from: NetworkAddress,
    data: Vec<u8>,
    protocols: Option<Vec<String>>,
}
