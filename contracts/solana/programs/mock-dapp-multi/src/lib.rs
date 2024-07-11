use anchor_lang::{
    prelude::*,
    solana_program::{hash, instruction::Instruction, program::invoke_signed},
};
use std::mem::size_of;

use xcall_lib::message::call_message::CallMessage;
use xcall_lib::message::call_message_persisted::CallMessagePersisted;
use xcall_lib::message::call_message_rollback::CallMessageWithRollback;
use xcall_lib::message::{msg_type::*, AnyMessage};
use xcall_lib::network_address::*;

use xcall_lib::message::envelope::Envelope;

declare_id!("Gj1bJC7rtUSYN8XZHpphDdTbqEGqsMPrRDGKhQJtFQyw");

#[program]

pub mod dapp_multi {
    use std::str::FromStr;

    use super::*;

    pub fn initialize(ctx: Context<InitializeCtx>, _xcall_address: Pubkey) -> Result<()> {
        ctx.accounts.config.set_inner(Config {
            xcall_address: _xcall_address,
            sn: 0,
        });
        Ok(())
    }

    pub fn send_call_message<'info>(
        ctx: Context<'_, '_, '_, 'info, CallMessageCtx<'info>>,
        to: NetworkAddress,
        data: Vec<u8>,
        msg_type: u32,
        rollback: Vec<u8>,
    ) -> Result<()> {
        let _xcall_address = ctx.accounts.config.xcall_address;
        let network_address = NetworkAddress::from(to);
        let _network_id = network_address.get_parts();

        msg!("network id: {}" , network_address.nid());

        let message = process_message(msg_type as u8, data, rollback).unwrap();
        let (sources, destinations) = get_network_connections(&ctx)?;

        let envelope = Envelope {
            message,
            sources: sources.clone(),
            destinations,
        };

        let encoded_envelope = rlp::encode(&envelope).to_vec();

        let ix_name = format!("{}:{}", "global", "send_call");
        let ix_discriminator = hash::hash(ix_name.as_bytes()).to_bytes()[..8].to_vec();

        let mut data = vec![];
    
        let args = SendMessageArgs {
            msg: encoded_envelope,
            to: network_address,
        };
        args.serialize(&mut data)?;

        let mut ix_data = Vec::new();
        ix_data.extend_from_slice(&ix_discriminator);
        ix_data.extend_from_slice(&data);

        let xcall_config = &ctx.remaining_accounts[0];
        let reply = &ctx.remaining_accounts[1];
        let rollback_account = &ctx.remaining_accounts[2];
        let default_connection = &ctx.remaining_accounts[3];
        let fee_handler = &ctx.remaining_accounts[4];
    
        // the accounts for centralized connections is contained here.
        let remaining_accounts = ctx.remaining_accounts.split_at(5).1;
        let mut account_metas: Vec<AccountMeta> = vec![
            // AccountMeta::new(ctx.accounts.config.key(), false), // signer
            AccountMeta::new(ctx.accounts.sender.key(), true), // signer
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false), // system program
            AccountMeta::new(xcall_config.key(), false), // xcall config
            AccountMeta::new(reply.key(), false), // reply
            AccountMeta::new(rollback_account.key(), false), // rollback
            AccountMeta::new_readonly(default_connection.key(), false), // default_connection
            AccountMeta::new(fee_handler.key(), false), // fee_handler
        ];
   

        let mut account_infos: Vec<AccountInfo> = vec![
            // ctx.accounts.config.to_account_info(),
            ctx.accounts.sender.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            xcall_config.to_account_info(),
            reply.to_account_info(),
            rollback_account.to_account_info(),
            default_connection.to_account_info(),
            fee_handler.to_account_info(),
        ];
        // adding centralized conection accounts in remaining account
        for accounts in remaining_accounts.iter().enumerate() {
            account_metas.push(AccountMeta::new(accounts.1.key(), accounts.1.is_signer));
            account_infos.push(accounts.1.to_account_info());
        }

        let ix = Instruction {
            program_id: _xcall_address,
            accounts: account_metas,
            data: ix_data.clone(),
        };
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];

        invoke_signed(&ix, &account_infos, signer_seeds)?;

        Ok(())
    }
   
    pub fn handle_call_message<'info>(
        ctx: Context<'_, '_, '_, 'info, CallMessageCtx<'info>>,
        from: NetworkAddress,
        data: Vec<u8>,
    ) -> Result<()> {
        let network_from = NetworkAddress::from(from.clone());

        let xcall_address = ctx.accounts.config.xcall_address;
        let (nid, account) = network_from.parse_network_address();

        if ctx.accounts.sender.key().to_string() == account {
            return Ok(());
        }

        let msg_data: String = rlp::decode(&data).unwrap(); // error handling remaining
        if msg_data == String::from("rollback") {
            msg!("Revert from dapp");
        } else {
            if msg_data == String::from("reply-response") {
                let message = AnyMessage::CallMessage(CallMessage {
                    data: "0xabc".try_to_vec().unwrap(),
                });

                if xcall_address.to_string().len() > 0 {
                    msg!("Un initialized");
                    return Ok(());
                }

                let (sources, destinations) = get_network_connections(&ctx).unwrap();

                let _envelope = Envelope {
                    message,
                    sources,
                    destinations,
                };

                let _xcall_address = ctx.accounts.config.xcall_address;
                let network_address = NetworkAddress::from(from);
                let _network_id = network_address.get_parts();
                let message = process_message(2, data.clone(), data).unwrap();
                let (sources, destinations) = get_network_connections(&ctx)?;

                let envelope = Envelope {
                    message,
                    sources: sources.clone(),
                    destinations,
                };

                let encoded_envelope = rlp::encode(&envelope).to_vec();

                let ix_name = format!("{}:{}", "global", "send_message");
                let ix_discriminator = hash::hash(ix_name.as_bytes()).to_bytes()[..8].to_vec();

                let mut data = vec![];
                let args = SendMessageArgs {
                    to: network_address,
                    msg: encoded_envelope,
                };
                args.serialize(&mut data)?;

                let mut ix_data = Vec::new();
                ix_data.extend_from_slice(&ix_discriminator);
                ix_data.extend_from_slice(&data);

                for (i, source) in sources.iter().enumerate() {
                    let config = &ctx.remaining_accounts[4 * i];
                    let reply = &ctx.remaining_accounts[1];
                    let rollback_account = &ctx.remaining_accounts[2];
                    let default_connection = &ctx.remaining_accounts[3];
                    let fee_handler = &ctx.remaining_accounts[3];

                    if source.to_owned() != default_connection.key().to_string() {
                        return Err(DappError::InvalidSource.into());
                    }

                    let account_metas: Vec<AccountMeta> = vec![
                        AccountMeta::new(ctx.accounts.sender.key(), true),
                        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                        AccountMeta::new(config.key(), false),
                        AccountMeta::new_readonly(reply.key(), false),
                        AccountMeta::new_readonly(rollback_account.key(), false),
                        AccountMeta::new_readonly(default_connection.key(), false),
                        AccountMeta::new_readonly(fee_handler.key(), false),
                    ];

                    let account_infos: Vec<AccountInfo> = vec![
                        ctx.accounts.sender.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                        config.to_account_info(),
                        reply.to_account_info(),
                        rollback_account.to_account_info(),
                        default_connection.to_account_info(),
                        fee_handler.to_account_info(),
                    ];
                    let ix = Instruction {
                        program_id: default_connection.key(),
                        accounts: account_metas,
                        data: ix_data.clone(),
                    };

                    let bump = ctx.bumps.config.clone();
                    invoke_signed(&ix, &account_infos, &[&[b"config", &[bump]]])?;
                }

                // let res = Self::xcall_send_call( &to, &envelope);
            }
        }

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
}

fn process_message(message_type: u8, data: Vec<u8>, rollback: Vec<u8>) -> Result<AnyMessage> {
    let msg_type: MessageType = message_type.into();

    let message = if msg_type == MessageType::CallMessagePersisted {
        AnyMessage::CallMessagePersisted(CallMessagePersisted { data })
    } else if msg_type == MessageType::CallMessageWithRollback {
        if rollback.len() > 0 {
            AnyMessage::CallMessageWithRollback(CallMessageWithRollback { data, rollback })
        } else {
            return Err(DappError::InvalidRollbackMessage.into());
        }
    } else {
        AnyMessage::CallMessage(CallMessage { data })
    };

    Ok(message)
}

fn get_network_connections(ctx: &Context<CallMessageCtx>) -> Result<(Vec<String>, Vec<String>)> {
    let connections = ctx.accounts.connections_account.connections.clone();

    let mut sources = Vec::new();
    let mut destinations = Vec::new();
    for conn in connections {
        sources.push(conn.src_endpoint);
        destinations.push(conn.dst_endpoint);
    }

    Ok((sources, destinations))
}

#[derive(Accounts)]
pub struct InitializeCtx<'info> {
    #[account(init , payer = sender , space= 8 + size_of::<Config>() , seeds=[b"config"] , bump )]
    config: Account<'info, Config>,

    #[account(mut)]
    sender: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub msg: Vec<u8>,
    pub to: NetworkAddress,
}

#[derive(Accounts)]
#[instruction(network_address: NetworkAddress )]
pub struct CallMessageCtx<'info> {
    #[account(mut , seeds=[b"config"] , bump )]
    config: Account<'info, Config>,

    // #[account(mut , seeds=[Connections::SEED_PREFIX.as_bytes()] , bump )]
    #[account(mut , seeds=[Connections::SEED_PREFIX.as_bytes(), network_address.nid().as_bytes()] , bump )]
    connections_account: Account<'info, Connections>,  // icon "icon"

    #[account(mut)]
    sender: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_network_id: String)]
pub struct AddConnectionCtx<'info> {
    #[account(init , payer = sender , space= Connections::MAX_SPACE , seeds=[Connections::SEED_PREFIX.as_bytes(), _network_id.as_bytes()] , bump )]
    connection_account: Account<'info, Connections>,

    #[account(mut)]
    sender: Signer<'info>,
    system_program: Program<'info, System>,
}

#[account]
pub struct Config {
    pub sn: u128,
    pub xcall_address: Pubkey,
}

#[account]
pub struct Connections {
    pub connections: Vec<Connection>,
}

impl Connections {
    pub const SEED_PREFIX: &'static str = "connections";
    pub const MAX_SPACE: usize = 4 + 256 + 4 + 256;
}

#[account]
#[derive(Debug)]
pub struct Connection {
    pub src_endpoint: String,
    pub dst_endpoint: String,
}

#[event]
pub struct MessageReceived {
    pub from: String,
    pub data: Vec<u8>,
}

#[event]
pub struct RollbackDataReceived {
    pub from: String,
    pub ssn: u128,
    pub rollback: Vec<u8>,
}

#[error_code]
pub enum DappError {
    #[msg("Address Mismatch")]
    AddressMismatch,
    #[msg("Rollback Mismatch")]
    RollbackMismatch,
    #[msg("Invalid Rollback Message")]
    InvalidRollbackMessage,
    #[msg("Uninitialized")]
    Uninitialized,
    #[msg("Invalid Source")]
    InvalidSource,
}
