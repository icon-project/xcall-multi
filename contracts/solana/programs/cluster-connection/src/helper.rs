use anchor_lang::{
    prelude::*,
    solana_program::{
        hash,
        instruction::{AccountMeta, Instruction},
        keccak::hashv,
        program::{get_return_data, invoke, invoke_signed},
        secp256k1_recover::secp256k1_recover,
        system_instruction,
    },
};

use crate::contexts::*;
use crate::error::*;
use crate::state::*;

use xcall_lib::{network_address::NetworkAddress, xcall_type};

pub const GET_NETWORK_ADDRESS: &str = "get_network_address";

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

pub fn get_message_hash(
    from_nid: &String,
    connection_sn: &u128,
    message: &Vec<u8>,
    dst_nid: &String,
) -> [u8; 32] {
    let mut encoded_bytes = Vec::new();
    encoded_bytes.extend(from_nid.as_bytes());
    encoded_bytes.extend(connection_sn.to_string().as_bytes());
    encoded_bytes.extend(message);
    encoded_bytes.extend(dst_nid.as_bytes());

    let hash = hashv(&[&encoded_bytes]);

    hash.to_bytes()
}

fn recover_pubkey(message: [u8; 32], sig: [u8; 65]) -> [u8; 64] {
    let recovery_id = sig[64] % 27;
    let signature = &sig[0..64];
    let recovered_pubkey = secp256k1_recover(&message, recovery_id, signature).unwrap();
    recovered_pubkey.to_bytes()
}

fn get_nid<'info>(ctx: &Context<'_, '_, '_, 'info, ReceiveMessageWithSignatures<'info>>) -> String {
    let ix_data = get_instruction_data(GET_NETWORK_ADDRESS, vec![]);
    let account_metas = vec![AccountMeta::new_readonly(
        ctx.remaining_accounts[1].key(),
        false,
    )];
    let mut account_infos = vec![];
    for i in ctx.remaining_accounts {
        account_infos.push(i.to_account_info());
    }

    let ix = Instruction {
        program_id: ctx.accounts.config.xcall,
        accounts: account_metas,
        data: ix_data,
    };

    invoke(&ix, &account_infos).unwrap();

    let network_address = NetworkAddress::try_from_slice(&get_return_data().unwrap().1).unwrap();
    return network_address.nid().to_string();
}

fn verify_signatures(
    from_nid: String,
    conn_sn: u128,
    message: Vec<u8>,
    dst_nid: String,
    signatures: Vec<[u8; 65]>,
    config: &Config,
) -> bool {
    let message_hash = get_message_hash(&from_nid, &conn_sn, &message, &dst_nid);
    let mut unique_validators = Vec::new();
    for sig in signatures {
        let pubkey = recover_pubkey(message_hash, sig);
        if !unique_validators.contains(&pubkey) && config.is_validator(&pubkey) {
            unique_validators.push(pubkey);
        }
    }

    if (unique_validators.len() as u8) < config.threshold {
        return false;
    }

    return true;
}

pub fn call_xcall_handle_message_with_signatures<'info>(
    ctx: Context<'_, '_, '_, 'info, ReceiveMessageWithSignatures<'info>>,
    from_nid: String,
    message: Vec<u8>,
    conn_sn: u128,
    sequence_no: u128,
    signatures: Vec<[u8; 65]>,
) -> Result<()> {
    let mut data = vec![];
    let dst_nid = get_nid(&ctx);

    if !verify_signatures(
        from_nid.clone(),
        conn_sn.clone(),
        message.clone(),
        dst_nid,
        signatures,
        &ctx.accounts.config,
    ) {
        return Err(ConnectionError::ValidatorsMustBeGreaterThanThreshold.into());
    }

    let args = xcall_type::HandleMessageArgs {
        from_nid,
        message,
        sequence_no,
        conn_sn,
    };
    args.serialize(&mut data)?;

    let ix_data = get_instruction_data("handle_message", data);

    invoke_instruction(
        ix_data,
        &ctx.accounts.config,
        &ctx.accounts.authority,
        &ctx.accounts.relayer,
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
        &ctx.accounts.relayer,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_decode<const N: usize>(s: &str) -> (Vec<u8>, [u8; N]) {
        let mut bytes = Vec::new();
        let mut i = 0;
        while i < s.len() {
            bytes.push(u8::from_str_radix(&s[i..i + 2], 16).unwrap());
            i += 2;
        }
        (bytes.clone(), bytes.try_into().unwrap_or_else(|_| [0; N]))
    }

    fn config() -> Config {
        Config::new(Pubkey::default(), Pubkey::default(), Pubkey::default(), 0)
    }

    #[test]
    fn test_recover_pubkey() {
        let from_nid = "0x2.icon";
        let connection_sn = 128;
        let message = b"hello";
        let dst_nid = "archway";

        let message_hash = get_message_hash(
            &from_nid.to_string(),
            &connection_sn,
            &message.to_vec(),
            &dst_nid.to_string(),
        );

        let signature = hex_decode::<65>("660d542b3f6de9cd08f238fd44133eeebfea290b21dae7322a63b516c57b8df12c4c0a340b60ed567c8da53578346c212b27b797eb42a75fb4b7076c567a6ff91c").1;
        let pubkey = recover_pubkey(message_hash, signature);
        assert_eq!(
            pubkey,
            hex_decode::<64>("deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209").1
        );
    }

    #[test]
    fn test_verify_signatures() {
        let from_nid = "0x2.icon".to_string();
        let conn_sn = 51;
        let message = hex_decode::<0>("c90287c682013101f800").0;
        let dst_nid = "solana-test".to_string();

        let val1: [u8; 65] = hex_decode::<65>("046bc928ee4932efd619ec4c00e0591e932cf2cfef13a59f6027da1c6cba36b35d91238b54aece19825025a9c7cb0bc58a60d5c49e7fc8e5b39fcc4c2193f5feb2").1;
        let signature: [u8; 65] = hex_decode::<65>("d28833bb4d03232378db9f3f9df8f53f11cb65a6b534aeb1a8792b4a01955ad17befde34e5195fae99a9b4c1ac76eb7ba95178af466ab357930603c0fc96a04e00").1;
        let signatures = vec![signature];
        let threshold = 1;

        let mut config = config();
        config.validators.push(val1);
        config.threshold = threshold;

        assert!(verify_signatures(
            from_nid, conn_sn, message, dst_nid, signatures, &config
        ));
    }

    #[test]
    fn test_verify_signatures_fail() {
        let from_nid = "0x2.icon".to_string();
        let conn_sn = 51;
        let message = hex_decode::<0>("c90287c682013101f800").0;
        let dst_nid = "solana-test".to_string();

        let val1: [u8; 65] = hex_decode::<65>("04deca512d5cb87673b23ab10c3e3572e30a2b5dc78cd500bf84bf066275c0bb320cb6cd266aa179b41323b5a18ab4a170248ed89b436b5559bab38c816ce12209").1;
        let val2: [u8; 65] = hex_decode::<65>("04f9379b2955d759a9532f8daa0c4a25da0ae706dd057de02af7754adb4b956ec9b8bf7a8a5a6686bc74dff736442a874c6bae5dcbcdb7113e24fbfa2337c63a01").1;

        let signatures  = vec![hex_decode::<65>("d28833bb4d03232378db9f3f9df8f53f11cb65a6b534aeb1a8792b4a01955ad17befde34e5195fae99a9b4c1ac76eb7ba95178af466ab357930603c0fc96a04e00").1, 
        hex_decode::<65>("d28833bb4d03232378db9f3f9df8f53f11cb65a6b534aeb1a8792b4a01955ad17befde34e5195fae99a9b4c1ac76eb7ba95178af466ab357930603c0fc96a04e00").1];

        let mut config = config();
        config.validators = vec![val1, val2];
        config.threshold = 1;

        assert!(!verify_signatures(
            from_nid, conn_sn, message, dst_nid, signatures, &config
        ));
    }
}
