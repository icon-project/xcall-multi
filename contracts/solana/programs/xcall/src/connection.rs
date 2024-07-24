use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{get_return_data, invoke, invoke_signed},
    },
};
use xcall_lib::xcall_connection_msg::{self, SEND_MESSAGE_IX};

use crate::{error::*, helper::get_instruction_data, state::*};

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

pub fn get_send_message_ix_data(to: &String, sn: i64, message: Vec<u8>) -> Result<Vec<u8>> {
    let mut data = vec![];
    let args = xcall_connection_msg::SendMessage {
        to: to.to_owned(),
        sn,
        msg: message,
    };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data(SEND_MESSAGE_IX, data);
    Ok(ix_data)
}
