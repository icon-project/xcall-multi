use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke_signed},
    },
};
use xcall_lib::{
    network_address::NetworkAddress,
    xcall_dapp_type::{
        self, HandleCallMessageResponse, HANDLE_CALL_MESSAGE_IX, QUERY_HANDLE_CALL_MESSAGE_IX,
    },
};

use crate::{error::XcallError, event, helper, state::*, types::result::CSResponseType};

/// Handles the response from the DApp after executing a call.
///
/// This function processes the response from the DApp's `handle_call_message` instruction,
/// determining whether the call was successful or not. It then emits an event indicating the
/// outcome of the call and returns the appropriate response type.
///
/// # Parameters
/// - `req_id`: The unique identifier of the request for which the call was executed.
/// - `dapp_res`: The response from the DApp, containing information about whether the call was
///   successful and any associated message.
///
/// # Returns
/// - `Result<CSResponseType>`: The response type indicating success or failure.
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

/// Invokes the `handle_call_message` instruction on a DApp program.
///
/// This function sends an instruction to the DApp program to handle a call message. It prepares
/// the required accounts and instruction data, invokes the instruction, and retrieves the response
/// from the DApp program.
///
/// # Parameters
/// - `dapp_key`: The public key of the DApp program that will handle the call message.
/// - `ix_data`: The serialized instruction data to be sent to the DApp program.
/// - `config`: The configuration account, used for verifying the program's settings and bump seed.
/// - `signer`: The account that signs the transaction, typically the one paying for it.
/// - `remaining_accounts`: A slice of account infos required by the DApp program, which will be
///   provided as additional accounts to the instruction.
///
/// # Returns
/// - `Result<xcall_dapp_type::HandleCallMessageResponse>`: The response from the DApp program,
///   indicating the result of handling the call message, or an error if the invocation fails.
pub fn invoke_handle_call_message_ix<'info>(
    dapp_key: Pubkey,
    ix_data: Vec<u8>,
    config: &Account<'info, Config>,
    signer: &Signer<'info>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<xcall_dapp_type::HandleCallMessageResponse> {
    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(signer.key(), true),
        AccountMeta::new_readonly(config.key(), true),
    ];
    let mut account_infos: Vec<AccountInfo<'info>> =
        vec![signer.to_account_info(), config.to_account_info()];
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
    let res = xcall_dapp_type::HandleCallMessageResponse::deserialize(&mut data_slice)?;

    Ok(res)
}

/// Prepares the instruction data to call the `handle_call_message` instruction on a DApp program.
///
/// This function constructs the instruction data required to invoke the `handle_call_message`
/// instruction on a DApp program. It serializes the message arguments, including the
/// from network address, data, and the protocols, into a format that the DApp program expects.
///
/// # Arguments
/// - `from`: A reference to the source network address.
/// - `data`: A vector of bytes representing the message payload.
/// - `protocols`: The list of protocols used to receive message in source and destination chain.
///
/// # Returns
/// - `Result<Vec<u8>>`: Serialized instruction data for the `handle_call_message` instruction.
pub fn get_handle_call_message_ix_data(
    from: NetworkAddress,
    data: Vec<u8>,
    protocols: Vec<String>,
) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = xcall_dapp_type::HandleCallMessageArgs {
        from,
        data,
        protocols,
    };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(HANDLE_CALL_MESSAGE_IX, ix_args_data);
    Ok(ix_data)
}

/// Prepares the instruction data to call the `query_handle_call_message_accounts` instruction
/// on DApp program.
///
/// This function constructs the instruction data to invoke the`query_handle_call_message_accounts`
/// instruction on a DApp program. It serializes the message arguments, including the
/// from network address, data, and the protocols, into a format that the DApp program expects.
///
/// # Arguments
/// - `from`: A reference to the source network address.
/// - `data`: A vector of bytes representing the message payload.
/// - `protocols`: The list of protocols used to receive message in source and destination chain.
///
/// # Returns
/// - `Result<Vec<u8>>`: Serialized instruction data for the `query_handle_call_message_accounts`
/// instruction.
pub fn get_query_handle_call_message_accounts_ix_data(
    from: NetworkAddress,
    data: Vec<u8>,
    protocols: Vec<String>,
) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = xcall_dapp_type::HandleCallMessageArgs {
        from,
        data,
        protocols,
    };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helper::get_instruction_data(QUERY_HANDLE_CALL_MESSAGE_IX, ix_args_data);
    Ok(ix_data)
}
