use std::ops::DerefMut;

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::{invoke, invoke_signed},
        system_instruction,
    },
};
use xcall_lib::{
    message::{envelope::Envelope, msg_trait::IMessage, AnyMessage},
    network_address::NetworkAddress,
    xcall_connection_msg::SendMessageArgs,
};

use crate::{
    error::XcallError,
    event, helper,
    state::*,
    types::{message::CSMessage, request::CSMessageRequest, rollback::Rollback},
};

pub fn send_call<'info>(
    ctx: Context<'_, '_, '_, 'info, SendCallCtx<'info>>,
    message: Vec<u8>,
    to: NetworkAddress,
) -> Result<u128> {
    let envelope: Envelope = rlp::decode(&message).unwrap();

    let signer = &ctx.accounts.signer;
    let config = ctx.accounts.config.deref_mut();
    let sequence_no = config.get_next_sn();

    let from = NetworkAddress::new(&config.network_id, &signer.key().to_string());

    process_message(
        &mut ctx.accounts.rollback_account,
        ctx.bumps.rollback_account,
        &signer,
        &to,
        &envelope,
    )?;

    let request = CSMessageRequest::new(
        from,
        to.account(),
        sequence_no,
        envelope.message.msg_type(),
        envelope.message.data(),
        envelope.destinations,
    );

    let need_response = request.need_response();

    let cs_message = CSMessage::from(request.clone());
    let encode_msg = cs_message.as_bytes();
    helper::ensure_data_length(&encode_msg)?;

    if is_reply(&ctx.accounts.reply, &to.nid(), &envelope.sources) && !need_response {
        ctx.accounts.reply.set_call_reply(request);
    } else {
        let sn = if need_response { sequence_no as i64 } else { 0 };

        let mut sources = envelope.sources;
        if sources.is_empty() {
            sources = vec![ctx.accounts.default_connection.key().to_string()]
        }

        let ix_discriminator = helper::get_instruction_discriminator("send_message");

        let mut data = vec![];
        let args = SendMessageArgs {
            to: to.nid(),
            sn,
            msg: encode_msg,
        };
        args.serialize(&mut data)?;

        let mut ix_data = Vec::new();
        ix_data.extend_from_slice(&ix_discriminator);
        ix_data.extend_from_slice(&data);

        for (i, source) in sources.iter().enumerate() {
            let connection = &ctx.remaining_accounts[4 * i];
            let config = &ctx.remaining_accounts[4 * i + 1];
            let network_fee = &ctx.remaining_accounts[4 * i + 2];
            let claim_fee = &ctx.remaining_accounts[4 * i + 3];

            if source.to_owned() != connection.key().to_string() {
                return Err(XcallError::InvalidSource.into());
            }

            let account_metas: Vec<AccountMeta> = vec![
                AccountMeta::new_readonly(ctx.accounts.reply.key(), true),
                AccountMeta::new(ctx.accounts.signer.key(), true),
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                AccountMeta::new(config.key(), false),
                AccountMeta::new_readonly(network_fee.key(), false),
                AccountMeta::new(claim_fee.key(), false),
            ];
            let account_infos: Vec<AccountInfo<'info>> = vec![
                ctx.accounts.reply.to_account_info(),
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                config.to_account_info(),
                network_fee.to_account_info(),
                claim_fee.to_account_info(),
            ];
            let ix = Instruction {
                program_id: connection.key(),
                accounts: account_metas,
                data: ix_data.clone(),
            };

            invoke_signed(
                &ix,
                &account_infos,
                &[&[Reply::SEED_PREFIX.as_bytes(), &[ctx.bumps.reply]]],
            )?;
        }

        // claim protocol fee
        if config.protocol_fee > 0 {
            claim_protocol_fee(
                &signer,
                &ctx.accounts.fee_handler,
                &ctx.accounts.system_program,
                config.protocol_fee,
            )?;
        }
    }

    emit!(event::CallMessageSent {
        from: signer.key(),
        to: to.to_string(),
        sn: sequence_no,
    });

    Ok(sequence_no)
}

pub fn process_message(
    rollback_account: &mut Option<Account<RollbackAccount>>,
    rollback_bump: Option<u8>,
    from: &AccountInfo,
    to: &NetworkAddress,
    envelope: &Envelope,
) -> Result<()> {
    match &envelope.message {
        AnyMessage::CallMessage(_) => Ok(()),
        AnyMessage::CallMessagePersisted(_) => Ok(()),
        AnyMessage::CallMessageWithRollback(msg) => {
            // TODO: remove comment -> temporary comment until testing from mock dapp
            // helper::ensure_program(from)?;
            helper::ensure_rollback_length(&msg.rollback)?;

            if msg.rollback().is_some() {
                let rollback_data = envelope.message.rollback().unwrap();
                let rollback = Rollback::new(
                    from.key(),
                    to.clone(),
                    envelope.sources.clone(),
                    rollback_data,
                    false,
                );

                let rollback_account = rollback_account
                    .as_mut()
                    .ok_or(XcallError::RollbackAccountNotSpecified)?;

                rollback_account.set(rollback, from.key(), rollback_bump.unwrap());
            }
            Ok(())
        }
    }
}

pub fn claim_protocol_fee<'info>(
    signer: &AccountInfo<'info>,
    fee_handler: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    protocol_fee: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(&signer.key(), &fee_handler.key(), protocol_fee);
    invoke(
        &ix,
        &[
            signer.to_owned(),
            fee_handler.to_owned(),
            system_program.to_owned(),
        ],
    )?;

    Ok(())
}

pub fn is_reply(reply: &Account<Reply>, nid: &String, sources: &Vec<String>) -> bool {
    if let Some(req) = &reply.reply_state {
        if req.from().nid() != *nid {
            return false;
        }
        return are_array_equal(req.protocols(), &sources);
    }
    false
}

pub fn are_array_equal(protocols: Vec<String>, sources: &Vec<String>) -> bool {
    if protocols.len() != sources.len() {
        return false;
    }
    for protocol in protocols.iter() {
        if !sources.contains(protocol) {
            return false;
        }
    }
    return true;
}

#[derive(Accounts)]
#[instruction(envelope: Vec<u8>, to: NetworkAddress)]
pub struct SendCallCtx<'info> {
    #[account(
        has_one = fee_handler,
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
      mut,
      seeds = [Reply::SEED_PREFIX.as_bytes()],
      bump
    )]
    pub reply: Account<'info, Reply>,

    #[account(
      init,
      payer = signer,
      space = RollbackAccount::SIZE,
      seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), (config.sequence_no + 1).to_string().as_bytes()],
      bump,
    )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), to.nid().as_bytes()],
        bump = default_connection.bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    /// CHECK: this is safe because we will verify if the protocol fee handler is valid or not
    #[account(mut)]
    pub fee_handler: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
