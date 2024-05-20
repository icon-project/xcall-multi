use std::str::FromStr;

use anchor_lang::solana_program::system_instruction;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::ErrorCode;
use crate::{CSMessageRequest, MessageType, NetworkAddress, ReplyData, XCallEnvelope, XCallState};

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;

const MAX_DATA_SIZE: u32 = 2048;
const DISCRIMINANT_END: usize = 8;

pub struct ProcessResult {
    pub need_response: bool,
    pub message: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FeesState {
    message_fees: u64,
    response_fees: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SendMessageArgs {
    pub to: String,
    pub sn: u128,
    pub msg: Vec<u8>,
}

#[event]
pub struct CallMessageSent {
    pub from: Pubkey,
    pub to: String,
    pub sn: u128,
}

#[derive(Accounts)]
pub struct SendMessageCtx<'info> {
    #[account(mut)]
    pub xcall_state: Account<'info, XCallState>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(mut)]
    pub reply_data: Account<'info, ReplyData>,

    /// CHECK: to transfer protocol fee
    #[account(mut)]
    pub fee_handler: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn send_message<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, SendMessageCtx<'info>>,
    to: String,
    msg: Vec<u8>,
) -> Result<()> {
    let envelope = XCallEnvelope::unmarshal_from(&msg).unwrap();
    let dst = NetworkAddress::split(to.clone());

    let new_seq = increment_sn(&mut ctx.accounts.xcall_state);
    let signer = ctx.accounts.sender.key();

    let msg_type = MessageType::from_int(envelope.msg_type.clone());
    let processed_msg = match msg_type {
        MessageType::CallMessageNormal => ProcessResult {
            need_response: false,
            message: envelope.message,
        },
        MessageType::CallMessagePersisted => ProcessResult {
            need_response: false,
            message: envelope.message,
        },
        _ => {
            require!(false, ErrorCode::UseSendMessageWithRollback);
            ProcessResult {
                need_response: false,
                message: vec![],
            }
        }
    };

    let xcall_state = &ctx.accounts.xcall_state;
    let system = ctx.accounts.system_program.to_account_info();

    let src_network = xcall_state.network_id.clone();

    let from = NetworkAddress::new(src_network, signer.to_string()).to_string();
    let msg_req = CSMessageRequest::new(
        from,
        to.clone(),
        new_seq,
        envelope.msg_type,
        processed_msg.message,
        envelope.destinations,
    );
    let msg_bytes = msg_req.as_bytes();
    require_gte!(MAX_DATA_SIZE, msg_bytes.len() as u32, ErrorCode::SizeExceed);

    let sources = envelope.sources;
    let signer_account = &ctx.accounts.sender;
    let system_program_info = &ctx.accounts.system_program.to_account_info();

    if is_reply(
        ctx.accounts.reply_data.reply_state.clone(),
        dst.net,
        sources.clone(),
    ) && !processed_msg.need_response
    {
        let reply_data = &mut ctx.accounts.reply_data;
        reply_data.call_reply = msg_bytes.clone();
        reply_data.reply_state = CSMessageRequest::null();
    } else {
        let send_sn = if processed_msg.need_response {
            new_seq
        } else {
            0
        };

        for (i, source) in sources.iter().enumerate() {
            let connection = &ctx.remaining_accounts[3 * i];
            let connection_fee = &ctx.remaining_accounts[3 * i + 1];
            let conn_fee = get_fee(connection_fee, send_sn as i128).unwrap();

            require_keys_eq!(
                *connection.owner,
                Pubkey::from_str(source.as_str()).unwrap()
            );

            // Transfer lamports to connection account
            invoke(
                &system_instruction::transfer(signer_account.key, connection.key, conn_fee),
                &[
                    signer_account.to_account_info(),
                    connection.clone(),
                    system_program_info.clone(),
                ],
            )?;

            // Call send message of connection program
            let args = SendMessageArgs {
                to: to.clone(),
                sn: new_seq,
                msg: msg_bytes.clone(),
            };
            let mut data = vec![];
            args.serialize(&mut data).unwrap();

            let mut instruction_data = Vec::new();
            instruction_data.extend_from_slice(&sighash("global", "send_message"));
            instruction_data.extend_from_slice(&data);

            let i = Instruction {
                program_id: *connection.owner,
                accounts: vec![
                    AccountMeta::new(connection.key(), false),
                    AccountMeta::new_readonly(connection_fee.key(), false),
                ],
                data: instruction_data,
            };

            let account_infos = vec![connection.clone(), connection_fee.clone(), system.clone()];

            let _ = invoke(&i, &account_infos);
        }
    }

    // claim protocol fee

    let protocol_fee = ctx.accounts.xcall_state.fee;

    let from_account = &ctx.accounts.sender;
    let fee_handler = &ctx.accounts.fee_handler;
    let fee_claim_instruction =
        system_instruction::transfer(from_account.key, fee_handler.key, protocol_fee);

    anchor_lang::solana_program::program::invoke(
        &fee_claim_instruction,
        &[
            from_account.to_account_info(),
            fee_handler.clone(),
            system_program_info.clone(),
        ],
    )?;

    Ok(())
}

pub fn increment_sn<'info>(state: &mut Account<'info, XCallState>) -> u128 {
    state.sequence_number += 1;
    state.sequence_number
}

pub fn get_fee(connection_fee: &AccountInfo, sn: i128) -> Result<u64> {
    if sn < 0 {
        return Ok(0);
    }
    let serialized_fee = &connection_fee.try_borrow_mut_data()?[DISCRIMINANT_END..];
    let conn_fee = FeesState::try_from_slice(&serialized_fee).unwrap();
    if sn > 0 {
        return Ok(conn_fee.message_fees + conn_fee.response_fees);
    }
    return Ok(conn_fee.message_fees);
}

pub fn is_reply(reply_data: CSMessageRequest, dst_net: String, sources: Vec<String>) -> bool {
    if reply_data.msg_type != u8::MAX {
        return NetworkAddress::split(reply_data.from).net == dst_net
            && protocol_equals(reply_data.protocols, sources);
    }
    false
}

pub fn protocol_equals(a: Vec<String>, b: Vec<String>) -> bool {
    a.len() == b.len() && a.iter().all(|item| b.contains(item))
}

pub fn sighash(namespace: &str, name: &str) -> Vec<u8> {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash.to_vec()
}

