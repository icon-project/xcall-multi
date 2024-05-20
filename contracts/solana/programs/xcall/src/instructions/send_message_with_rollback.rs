use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;

use crate::{get_fee, increment_sn, is_reply, sighash, 
    CSMessageRequest, CallMessageWithRollback, MessageType, NetworkAddress, 
    ProcessResult, ReplyData, RollbackData, SendMessageArgs, XCallEnvelope, XCallState};
use crate::error::ErrorCode;


const MAX_DATA_SIZE: u32 = 2048;

#[account]
pub struct RollbackDataState {
    pub sequence_number: u128,
    pub data: RollbackData,
}

#[derive(Accounts)]
#[instruction(sequence_number: u128)]
pub struct SendMessageWithRollbackCtx<'info> {
    #[account(
        init, 
        payer = sender, 
        space = 8 + 512,
    )]
    pub rollback_data_state: Account<'info, RollbackDataState>,

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

pub fn send_message_with_rollback<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, SendMessageWithRollbackCtx<'info>>,
    to: String,
    msg: Vec<u8>,
) -> Result<()> {
    let envelope = XCallEnvelope::unmarshal_from(&msg).unwrap();
    let dst = NetworkAddress::split(to.clone());

    let new_seq = increment_sn(&mut ctx.accounts.xcall_state);

    let signer = ctx.accounts.sender.key();

    let msg_type = MessageType::from_int(envelope.msg_type.clone());
    let processed_msg = match msg_type {
        MessageType::CallMessageWithRollback => {
            let call_message = CallMessageWithRollback::unmarshal_from(&envelope.message).unwrap();

            // todo: require signer to be a program, and not user, maybe limit what seed the program should use to call this method?

            let rollback = RollbackData::new(signer, dst.net.clone(), envelope.sources.clone(), call_message.rollback, true);

            let rollback_acc = &mut ctx.accounts.rollback_data_state;
            rollback_acc.data = rollback;
            rollback_acc.sequence_number = new_seq;

            ProcessResult {
                need_response: true,
                message: call_message.data,
            }
        },
        _ => {
              require!(false, ErrorCode::UseSendMessage);
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

    let sources = envelope.sources.clone();
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