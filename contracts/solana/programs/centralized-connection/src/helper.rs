use anchor_lang::{
    prelude::*,
    solana_program::{
        hash,
        instruction::Instruction,
        program::{invoke, invoke_signed},
        system_instruction,
    },
};

use crate::contexts::*;
use crate::state::*;

use xcall_lib::xcall_msg::{HandleError, HandleMessage};

pub fn transfer_lamports<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(&from.key(), &to.key(), amount);
    invoke(
        &ix,
        &[from.to_owned(), to.to_owned(), system_program.to_owned()],
    )?;

    Ok(())
}

pub fn get_instruction_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", "global", name);

    let mut ix_discriminator = [0u8; 8];
    ix_discriminator.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);

    ix_discriminator
}

pub fn call_xcall_handle_message<'info>(
    ctx: Context<'_, '_, '_, 'info, RecvMessage<'info>>,
    from_nid: String,
    message: Vec<u8>,
    sequence_no: u128,
) -> Result<()> {
    let mut data = vec![];
    let args = HandleMessage {
        from_nid,
        message,
        sequence_no,
    };
    args.serialize(&mut data)?;

    invoke_instruction(
        "handle_message",
        data,
        &ctx.accounts.config,
        &ctx.accounts.admin,
        &ctx.accounts.system_program,
        ctx.remaining_accounts,
    )
}

pub fn call_xcall_handle_error<'info>(
    ctx: Context<'_, '_, '_, 'info, RevertMessage<'info>>,
    from_nid: String,
    sequence_no: u128,
) -> Result<()> {
    let mut data = vec![];
    let args = HandleError {
        from_nid,
        sequence_no,
    };
    args.serialize(&mut data)?;

    invoke_instruction(
        "handle_error",
        data,
        &ctx.accounts.config,
        &ctx.accounts.admin,
        &ctx.accounts.system_program,
        &ctx.remaining_accounts,
    )
}

pub fn invoke_instruction<'info>(
    ix_name: &str,
    ix_args: Vec<u8>,
    config: &Account<'info, Config>,
    admin: &Signer<'info>,
    system_program: &Program<'info, System>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    let ix_discriminator = get_instruction_discriminator(ix_name);

    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&ix_discriminator);
    ix_data.extend_from_slice(&ix_args);

    let mut account_metas = vec![
        AccountMeta::new_readonly(config.key(), true),
        AccountMeta::new(admin.key(), true),
        AccountMeta::new_readonly(system_program.key(), false),
    ];
    let mut account_infos = vec![
        config.to_account_info(),
        admin.to_account_info(),
        system_program.to_account_info(),
    ];
    for i in remaining_accounts {
        account_metas.push(AccountMeta::new(i.key(), i.is_signer));
        account_infos.push(i.to_account_info());
    }
    let ix = Instruction {
        program_id: config.xcall,
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
