use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke, invoke_signed},
    },
};
use xcall_lib::xcall_connection_type::{self, QUERY_SEND_MESSAGE_ACCOUNTS_IX, SEND_MESSAGE_IX};

use crate::{error::*, helper::get_instruction_data, state::*};

/// Queries the fee for sending a message through a connection program.
///
/// This function sends an instruction to the connection program, querying the network fee for sending
/// a message. It prepares the required accounts and instruction data, invokes the program, and
/// retrieves the fee from the program's response.
///
/// # Parameters
/// - `source`: A reference to a string representing the connection's public key as a string.
/// - `ix_data`: A reference to a vector containing the serialized instruction data.
/// - `network_fee_account`: A reference to the account that holds the network fee.
///
/// # Returns
/// - `Result<u64>`: The fee required by the connection program, or an error if the query fails.
pub fn query_connection_fee<'info>(
    source: &String,
    ix_data: &Vec<u8>,
    network_fee_account: &AccountInfo,
) -> Result<u64> {
    let connection = Pubkey::from_str(&source).map_err(|_| XcallError::InvalidPubkey)?;

    let account_metas = vec![AccountMeta::new(network_fee_account.key(), false)];
    let account_infos = vec![network_fee_account.to_account_info()];

    let ix = Instruction {
        program_id: connection,
        accounts: account_metas,
        data: ix_data.clone(),
    };

    invoke(&ix, &account_infos)?;

    let (_, fee) = get_return_data().unwrap();
    let fee = u64::deserialize(&mut fee.as_ref())?;

    Ok(fee)
}

/// Sends a cross-chain message to a connection program.
///
/// This function constructs and sends an `Instruction` to a connection program, using the provided
/// configuration and remaining accounts. It handles the indexing of accounts based on the
/// specified `index`, prepares the necessary `AccountMeta` and `AccountInfo` objects, and then
/// invokes the instruction
///
/// # Arguments
/// - `index`: The index of the connection-related accounts in the `remaining_accounts` array.
///   Each connection requires three accounts: connection, connection configuration, and network fee
/// - `ix_data`: A vector of bytes representing the serialized instruction data to be sent.
/// - `sysvar_account`: The instruction sysvar account, used to verify if the current instruction is a
/// program invocation.
/// - `signer`: The account that signs the transaction.
/// - `system_program`: The system program account.
/// - `remaining_accounts`: A slice of `AccountInfo` containing additional accounts required
///   for the connection, including the connection program, connection configuration, and network fee
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the message was successfully sent, otherwise returns an error.
pub fn call_connection_send_message<'info>(
    index: usize,
    ix_data: &Vec<u8>,
    config: &Account<'info, Config>,
    signer: &Signer<'info>,
    system_program: &Program<'info, System>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    let connection = &remaining_accounts[3 * index];
    let conn_config = &remaining_accounts[3 * index + 1];
    let network_fee = &remaining_accounts[3 * index + 2];

    let account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(signer.key(), true),
        AccountMeta::new_readonly(system_program.key(), false),
        AccountMeta::new_readonly(config.key(), true),
        AccountMeta::new(conn_config.key(), false),
        AccountMeta::new_readonly(network_fee.key(), false),
    ];
    let account_infos: Vec<AccountInfo<'info>> = vec![
        conn_config.to_account_info(),
        signer.to_account_info(),
        system_program.to_account_info(),
        config.to_account_info(),
        network_fee.to_account_info(),
    ];
    let ix = Instruction {
        program_id: connection.key(),
        accounts: account_metas,
        data: ix_data.clone(),
    };

    invoke_signed(
        &ix,
        &account_infos,
        &[&[Config::SEED_PREFIX.as_bytes(), &[config.bump]]],
    )?;

    Ok(())
}

/// Prepares the instruction data to call the `send_message` instruction on a connection program.
///
/// This function constructs the instruction data required to invoke the `send_message` instruction
/// on a specified connection (source) program. It serializes the message arguments, including the
/// destination network address, sequence number, and the message payload, into a format that the
/// connection program expects.
///
/// # Arguments
/// - `to`: A reference to the destination network address.
/// - `sn`: The sequence number associated with the message.
/// - `message`: A vector of bytes representing the message payload.
///
/// # Returns
/// - `Result<Vec<u8>>`: Serialized instruction data for the `send_message` instruction.
/// Returns an error if serialization fails.
pub fn get_send_message_ix_data(to: &String, sn: i64, message: Vec<u8>) -> Result<Vec<u8>> {
    let mut data = vec![];
    let args = xcall_connection_type::SendMessageArgs {
        to: to.to_owned(),
        sn,
        msg: message,
    };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data(SEND_MESSAGE_IX, data);
    Ok(ix_data)
}

/// Prepares the instruction data to query send message accounts of a connection program.
///
/// This function constructs the data required to invoke the `query_send_message_accounts`
/// instruction on a connection program. It serializes the message arguments, including the
/// destination network address, sequence number, and the message payload, into the expected format.
///
/// # Arguments
/// - `to`: A reference to the destination network address.
/// - `sn`: The sequence number associated with the message.
/// - `message`: A vector of bytes representing the message payload.
///
/// # Returns
/// - `Result<Vec<u8>>`: Serialized instruction data for the `query_send_message_accounts`
///   instruction. Returns an error if serialization fails.
pub fn get_query_send_message_accounts_ix_data(
    to: &String,
    sn: i64,
    message: Vec<u8>,
) -> Result<Vec<u8>> {
    let mut data = vec![];
    let args = xcall_connection_type::SendMessageArgs {
        to: to.to_owned(),
        sn,
        msg: message,
    };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data(QUERY_SEND_MESSAGE_ACCOUNTS_IX, data);
    Ok(ix_data)
}
