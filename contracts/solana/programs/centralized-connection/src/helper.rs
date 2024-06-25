use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};

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
