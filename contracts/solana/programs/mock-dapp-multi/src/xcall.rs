use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use xcall_lib::{
    network_address::NetworkAddress,
    xcall_type::{self, SEND_CALL_IX},
};

use crate::{helpers, Config};

pub fn call_xcall_send_call<'info>(
    ix_data: &Vec<u8>,
    config: &Account<'info, Config>,
    signer: &Signer<'info>,
    system_program: &Program<'info, System>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(signer.key(), true),
        AccountMeta::new_readonly(system_program.key(), false),
    ];
    let mut account_infos: Vec<AccountInfo<'info>> =
        vec![signer.to_account_info(), system_program.to_account_info()];
    for (_, account) in remaining_accounts.iter().enumerate() {
        if account.is_writable {
            account_metas.push(AccountMeta::new(account.key(), account.is_signer))
        } else {
            account_metas.push(AccountMeta::new_readonly(account.key(), account.is_signer))
        }
        account_infos.push(account.to_account_info());
    }
    let ix = Instruction {
        program_id: config.xcall_address,
        accounts: account_metas,
        data: ix_data.clone(),
    };

    invoke(&ix, &account_infos)?;

    Ok(())
}

pub fn get_send_call_ix_data(msg: Vec<u8>, to: NetworkAddress) -> Result<Vec<u8>> {
    let mut ix_args_data = vec![];
    let ix_args = xcall_type::SendCallArgs { msg, to };
    ix_args.serialize(&mut ix_args_data)?;

    let ix_data = helpers::get_instruction_data(SEND_CALL_IX, ix_args_data);
    Ok(ix_data)
}