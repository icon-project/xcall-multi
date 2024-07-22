use std::vec;

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke_signed},
    },
};
use xcall_lib::{
    network_address::NetworkAddress,
    xcall_dapp_msg::{self, HandleCallMessageResponse, HANDLE_CALL_MESSAGE_IX},
};

use crate::{error::XcallError, event, helper, state::*, types::result::CSResponseType};

pub fn handle_response(
    req_id: u128,
    dapp_res: HandleCallMessageResponse,
) -> Result<CSResponseType> {
    let res_code = match dapp_res.success {
        true => CSResponseType::CSResponseSuccess,
        false => CSResponseType::CSResponseFailure,
    };

    emit!(event::CallExecuted {
        reqId: req_id,
        code: res_code.clone().into(),
        msg: dapp_res.message,
    });

    Ok(res_code)
}

pub fn invoke_handle_call_message_ix<'info>(
    dapp_key: Pubkey,
    ix_data: Vec<u8>,
    config: &Account<'info, Config>,
    signer: &Signer<'info>,
    system_program: &Program<'info, System>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<xcall_dapp_msg::HandleCallMessageResponse> {
    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(signer.key(), true),
        AccountMeta::new_readonly(system_program.key(), false),
    ];
    let mut account_infos: Vec<AccountInfo<'info>> =
        vec![signer.to_account_info(), system_program.to_account_info()];
    for account in remaining_accounts {
        if account.is_writable {
            account_metas.push(AccountMeta::new(account.key(), account.is_signer))
        } else {
            account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer))
        }
        account_infos.push(account.to_account_info());
    }
    let ix = Instruction {
        program_id: dapp_key,
        accounts: account_metas,
        data: ix_data,
    };

    invoke_signed(
        &ix,
        &account_infos,
        &[&[Config::SEED_PREFIX.as_bytes(), &[config.bump]]],
    )?;

    let (_, data) = get_return_data().ok_or(XcallError::InvalidResponse)?;
    let mut data_slice: &[u8] = &data;
    let res = xcall_dapp_msg::HandleCallMessageResponse::deserialize(&mut data_slice)?;

    Ok(res)
}

pub fn get_handle_call_message_ix_data(
    from: NetworkAddress,
    data: Vec<u8>,
    protocols: Option<Vec<String>>,
) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = xcall_dapp_msg::HandleCallMessage {
        from,
        data,
        protocols,
    };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(HANDLE_CALL_MESSAGE_IX, ix_args_data);
    Ok(ix_data)
}
