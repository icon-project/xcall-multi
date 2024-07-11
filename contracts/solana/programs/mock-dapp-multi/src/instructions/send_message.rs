use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use std::mem::size_of;

use xcall_lib::network_address::*;

use xcall_lib::message::envelope::Envelope;

use crate::{
    get_instruction_data, get_network_connections, process_message, Config, Connection, Connections,
};

pub fn send_message<'info>(
    ctx: Context<'_, '_, '_, 'info, CallMessageCtx<'info>>,
    to: NetworkAddress,
    data: Vec<u8>,
    msg_type: u32,
    rollback: Vec<u8>,
) -> Result<()> {
    let _xcall_address = ctx.accounts.config.xcall_address;
    let network_address = NetworkAddress::from(to);
    let _network_id = network_address.get_parts();

    let message = process_message(msg_type as u8, data, rollback).unwrap();
    let (sources, destinations) = get_network_connections(&ctx)?;

    let envelope = Envelope {
        message,
        sources: sources.clone(),
        destinations,
    };

    let encoded_envelope = rlp::encode(&envelope).to_vec();

    let mut data = vec![];

    let args = SendMessageArgs {
        msg: encoded_envelope,
        to: network_address,
    };
    args.serialize(&mut data)?;
    let ix_data = get_instruction_data("send_call", data);

    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(ctx.accounts.sender.key(), true), // signer
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false), // system program
    ];

    let mut account_infos: Vec<AccountInfo> = vec![
        ctx.accounts.sender.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];

    for (_index, account) in ctx.remaining_accounts.iter().enumerate() {
        account_metas.push(AccountMeta::new(account.key(), account.is_signer));
        account_infos.push(account.to_account_info());
    }

    let ix = Instruction {
        program_id: _xcall_address,
        accounts: account_metas,
        data: ix_data.clone(),
    };
    let signer_seeds: &[&[&[u8]]] = &[&[Config::SEED_PREFIX.as_bytes(), &[ctx.bumps.config]]];

    invoke_signed(&ix, &account_infos, signer_seeds)?;

    Ok(())
}

pub fn add_connection(
    ctx: Context<AddConnectionCtx>,
    _network_id: String,
    src_endpoint: String,
    dst_endpoint: String,
) -> Result<()> {
    ctx.accounts
        .connection_account
        .connections
        .push(Connection {
            src_endpoint,
            dst_endpoint,
        });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeCtx<'info> {
    #[account(init , payer = sender , space= 8 + size_of::<Config>() , seeds=[Config::SEED_PREFIX.as_bytes()] , bump )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub msg: Vec<u8>,
    pub to: NetworkAddress,
}

#[derive(Accounts)]
#[instruction(network_address: NetworkAddress )]
pub struct CallMessageCtx<'info> {
    #[account(mut , seeds=[Config::SEED_PREFIX.as_bytes()] , bump )]
    pub config: Account<'info, Config>,

    #[account(mut , seeds=[Connections::SEED_PREFIX.as_bytes(), network_address.nid().as_bytes()] , bump )]
    pub connections_account: Account<'info, Connections>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_network_id: String)]
pub struct AddConnectionCtx<'info> {
    #[account(init , payer = sender , space= Connections::MAX_SPACE , seeds=[Connections::SEED_PREFIX.as_bytes(), _network_id.as_bytes()] , bump )]
    pub connection_account: Account<'info, Connections>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}
