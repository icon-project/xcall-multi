use anchor_lang::{
    prelude::*,
    solana_program::{
        hash,
        instruction::{AccountMeta,Instruction},
        program::{invoke, invoke_signed},
        sysvar::{recent_blockhashes::ID as SYSVAR_ID},
        ed25519_program,
        system_instruction,
    },
};

use ed25519_dalek::{PublicKey as Ed25519PublicKey, Signature, Verifier};
use crate::contexts::*;
use crate::state::*;

use xcall_lib::xcall_type;

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

pub fn get_instruction_data(ix_name: &str, data: Vec<u8>) -> Vec<u8> {
    let preimage = format!("{}:{}", "global", ix_name);

    let mut ix_discriminator = [0u8; 8];
    ix_discriminator.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);

    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&ix_discriminator);
    ix_data.extend_from_slice(&data);

    ix_data
}

pub fn verify_message(signature: [u8; 64], message: Vec<u8>, pubkey: Pubkey) -> Result<()> {
    // Recover the public key from the signature

    // let pubkey = Pubkey::from([
    //     57, 234, 243, 206, 86, 29, 102, 46, 179, 39, 246, 137, 159, 74, 167, 40, 
    //     69, 191, 199, 163, 68, 114, 221, 40, 45, 129, 56, 73, 87, 58, 119, 149
    // ]);

    let validator_pubkey = Ed25519PublicKey::from_bytes(&pubkey.to_bytes()).unwrap();

    let signature = Signature::from_bytes(&signature).unwrap();

    validator_pubkey
        .verify(&message, &signature)
        .unwrap();
    

    Ok(())
}

pub fn call_xcall_handle_message<'info>(
    ctx: Context<'_, '_, '_, 'info, RecvMessage<'info>>,
    from_nid: String,
    message: Vec<u8>,
    sequence_no: u128,
) -> Result<()> {
    let mut data = vec![];
    let args = xcall_type::HandleMessageArgs {
        from_nid,
        message,
        sequence_no,
    };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data("handle_message", data);

    invoke_instruction(
        ix_data,
        &ctx.accounts.config,
        &ctx.accounts.authority,
        &ctx.accounts.admin,
        &ctx.accounts.system_program,
        ctx.remaining_accounts,
    )
}

pub fn call_xcall_handle_message_with_signatures<'info>(
    ctx: Context<'_, '_, '_, 'info, ReceiveMessageWithSignatures<'info>>,
    from_nid: String,
    message: Vec<u8>,
    sequence_no: u128,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    let mut data = vec![];
    let args = xcall_type::HandleMessageArgs {
        from_nid,
        message,
        sequence_no,
    };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data("handle_message", data);

    invoke_instruction(
        ix_data,
        &ctx.accounts.config,
        &ctx.accounts.authority,
        &ctx.accounts.admin,
        &ctx.accounts.system_program,
        ctx.remaining_accounts,
    )
}

pub fn call_xcall_handle_error<'info>(
    ctx: Context<'_, '_, '_, 'info, RevertMessage<'info>>,
    sequence_no: u128,
) -> Result<()> {
    let mut data = vec![];
    let args = xcall_type::HandleErrorArgs { sequence_no };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data("handle_error", data);

    invoke_instruction(
        ix_data,
        &ctx.accounts.config,
        &ctx.accounts.authority,
        &ctx.accounts.admin,
        &ctx.accounts.system_program,
        &ctx.remaining_accounts,
    )
}

pub fn invoke_instruction<'info>(
    ix_data: Vec<u8>,
    config: &Account<'info, Config>,
    authority: &Account<'info, Authority>,
    admin: &Signer<'info>,
    system_program: &Program<'info, System>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    let mut account_metas = vec![
        AccountMeta::new(admin.key(), true),
        AccountMeta::new_readonly(authority.key(), true),
        AccountMeta::new_readonly(system_program.key(), false),
    ];
    let mut account_infos = vec![
        admin.to_account_info(),
        authority.to_account_info(),
        system_program.to_account_info(),
    ];
    for i in remaining_accounts {
        if i.is_writable {
            account_metas.push(AccountMeta::new(i.key(), i.is_signer));
        } else {
            account_metas.push(AccountMeta::new_readonly(i.key(), i.is_signer))
        }
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
        &[&[Authority::SEED_PREFIX.as_bytes(), &[authority.bump]]],
    )?;

    Ok(())
}

#[test]
fn test_verify_message() {
    let message: Vec<u8> = b"Test message".to_vec();
    let result = verify_message([
        73, 220,  38, 229, 202, 199, 238, 241,  69, 237, 194,
       226, 113,  76,  59, 167, 134,  83,  13, 213, 245, 151,
        26, 220, 196, 162, 247, 240,  95, 245,  78, 205,  26,
       233, 140, 177, 151, 207, 193,  23,  71,  96, 126, 115,
       200,  21, 108, 178,  48, 133, 117, 208,  27,  80, 237,
        93, 180,  97, 235,  40, 109,  36,  31,  11
     ], message.clone(), Pubkey::new(&[57, 234, 243, 206, 86, 29, 102, 46, 179, 39, 246, 137, 159, 74, 167, 40, 
        69, 191, 199, 163, 68, 114, 221, 40, 45, 129, 56, 73, 87, 58, 119, 149]
       ));

    // Step 6: Assert that the signature verification is successful
    assert!(result.is_ok(), "Signature verification failed");
}
