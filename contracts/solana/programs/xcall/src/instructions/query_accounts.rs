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
    account_metadata::AccountMetadata, network_address::NetworkAddress,
    xcall_msg::QueryAccountsResponse,
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

pub fn query_execute_call_accounts(
    ctx: Context<QueryExecuteCallAccountsCtx>,
    req_id: u128,
    data: Vec<u8>,
) -> Result<QueryAccountsResponse> {
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
        let connection = Pubkey::from_str(&source).map_err(|_| XcallError::InvalidPubkey)?;
        let conn_config = &ctx.remaining_accounts[i];

        let account_metas = vec![AccountMeta::new(conn_config.key(), false)];
        let account_infos = vec![conn_config.to_account_info()];
        let ix = Instruction {
            program_id: connection,
            accounts: account_metas,
            data: conn_ix_data.clone(),
        };

        invoke(&ix, &account_infos)?;

        let (_, data) = get_return_data().unwrap();
        let mut data_slice: &[u8] = &data;
        let res = QueryAccountsResponse::deserialize(&mut data_slice)?;

        let mut res_accounts = res.accounts;
        account_metadata.push(AccountMetadata::new(connection, false));
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

    let mut dapp_account_metas: Vec<AccountMeta> = vec![];
    let mut dapp_account_infos: Vec<AccountInfo> = vec![];

    let remaining_accounts = &ctx.remaining_accounts[(protocols.len())..];
    for (_, account) in remaining_accounts.iter().enumerate() {
        if account.is_writable {
            dapp_account_metas.push(AccountMeta::new(account.key(), account.is_signer));
        } else {
            dapp_account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer));
        }

        dapp_account_infos.push(account.to_account_info())
    }

    let dapp_key = Pubkey::from_str(proxy_req.to()).map_err(|_| XcallError::InvalidPubkey)?;

    let dapp_ix = Instruction {
        program_id: dapp_key,
        accounts: dapp_account_metas,
        data: dapp_ix_data,
    };

    invoke(&dapp_ix, &dapp_account_infos)?;

    let (_, data) = get_return_data().unwrap();
    let mut data_slice: &[u8] = &data;
    let res = QueryAccountsResponse::deserialize(&mut data_slice)?;

    let mut res_accounts = res.accounts;
    account_metadata.append(&mut res_accounts);

    Ok(QueryAccountsResponse {
        accounts: account_metadata,
    })
}

pub fn get_query_send_message_accounts_ix_data(dst_network: String) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = QuerySendMessage { to: dst_network };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data("query_send_message_accounts", ix_args_data);
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

    let ix_data = helper::get_instruction_data("query_handle_call_message_accounts", ix_args_data);
    Ok(ix_data)
}

pub fn query_handle_message_accounts(
    ctx: Context<QueryAccountsCtx>,
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
            let (rollback_account, _) = Pubkey::find_program_address(
                &[
                    RollbackAccount::SEED_PREFIX.as_bytes(),
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

            if ctx.accounts.rollback_account.rollback.protocols().len() > 1 {
                account_metas.push(AccountMetadata::new(pending_response, false));
            } else {
                account_metas.push(AccountMetadata::new(id(), false))
            }

            if result.response_code() == &CSResponseType::CSResponseSuccess {
                account_metas.push(AccountMetadata::new(successful_response, false))
            } else {
                account_metas.push(AccountMetadata::new(id(), false));
            }

            account_metas.push(AccountMetadata::new(rollback_account, false));
        }
    }

    Ok(QueryAccountsResponse {
        accounts: account_metas,
    })
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
pub struct QueryAccountsCtx<'info> {
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sequence_no.to_be_bytes()],
        bump = rollback_account.bump
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
