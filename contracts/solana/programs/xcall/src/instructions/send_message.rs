use anchor_lang::{
    prelude::*,
    solana_program::{
        program::invoke,
        system_instruction,
        sysvar::{self, instructions::get_instruction_relative},
    },
};
use xcall_lib::{
    message::{envelope::Envelope, msg_trait::IMessage, AnyMessage},
    network_address::NetworkAddress,
};

use crate::{
    connection,
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

    let sequence_no = ctx.accounts.config.get_next_sn();

    let signer = &ctx.accounts.signer;
    let config = &ctx.accounts.config;

    let sysvar_account = &ctx.accounts.instruction_sysvar.to_account_info();
    let current_ix = get_instruction_relative(0, sysvar_account)?;

    let from_key = if current_ix.program_id != crate::id() {
        current_ix.program_id
    } else {
        signer.key()
    };

    let from = NetworkAddress::new(&config.network_id, &from_key.to_string());

    process_message(
        &mut ctx.accounts.rollback_account,
        &ctx.accounts.instruction_sysvar,
        ctx.bumps.rollback_account,
        from_key,
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

    let cs_message = CSMessage::from(request.clone()).as_bytes();
    helper::ensure_data_length(&cs_message)?;

    let sources = envelope.sources;
    if sources.is_empty() {
        return Err(XcallError::ProtocolNotSpecified.into());
    }

    if is_reply(&config, &to.nid(), &sources) && !need_response {
        ctx.accounts.config.set_call_reply(Some(request));
    } else {
        let sn: i64 = if need_response { sequence_no as i64 } else { 0 };
        let ix_data = connection::get_send_message_ix_data(&to.nid(), sn, cs_message)?;

        for (i, _) in sources.iter().enumerate() {
            connection::call_connection_send_message(
                i,
                &ix_data,
                &ctx.accounts.config,
                &ctx.accounts.signer,
                &ctx.accounts.system_program,
                &ctx.remaining_accounts,
            )?;
        }

        if config.protocol_fee > 0 {
            claim_protocol_fee(
                &ctx.accounts.signer,
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
    sysvar_account_info: &AccountInfo,
    rollback_bump: Option<u8>,
    from: Pubkey,
    to: &NetworkAddress,
    envelope: &Envelope,
) -> Result<()> {
    match &envelope.message {
        AnyMessage::CallMessage(_) => Ok(()),
        AnyMessage::CallMessagePersisted(_) => Ok(()),
        AnyMessage::CallMessageWithRollback(msg) => {
            helper::ensure_rollback_size(&msg.rollback)?;
            helper::ensure_program(sysvar_account_info)?;

            let rollback = Rollback::new(
                from,
                to.clone(),
                envelope.sources.clone(),
                msg.rollback().unwrap(),
                false,
            );

            let rollback_account = rollback_account
                .as_mut()
                .ok_or(XcallError::RollbackAccountNotSpecified)?;

            rollback_account.set(rollback, rollback_bump.unwrap());

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

pub fn is_reply(config: &Account<Config>, nid: &String, sources: &Vec<String>) -> bool {
    if let Some(req) = &config.reply_state {
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
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        has_one = fee_handler,
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: this is safe because we will verify if the protocol fee handler is valid or not
    #[account(mut)]
    pub fee_handler: AccountInfo<'info>,

    #[account(
        init,
        payer = signer,
        space = RollbackAccount::SIZE,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &(config.sequence_no + 1).to_be_bytes()],
        bump,
      )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,

    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar: UncheckedAccount<'info>,
}
