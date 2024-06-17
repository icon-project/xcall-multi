use anchor_lang::prelude::*;
use xcall_lib::{
    message::{envelope::Envelope, msg_trait::IMessage, AnyMessage},
    network_address::{NetId, NetworkAddress},
};

use std::ops::DerefMut;

use crate::{
    constants,
    error::XcallError,
    state::*,
    types::{message::CSMessage, request::CSMessageRequest, rollback::Rollback},
};

pub fn send_call(
    ctx: Context<SendCallCtx>,
    envelope: Envelope,
    to: NetworkAddress,
) -> Result<u128> {
    let signer = &ctx.accounts.signer;
    let config = ctx.accounts.config.deref_mut();
    let sequence_no = config.get_next_sn();

    let from = NetworkAddress::new(&config.network_id, &signer.key().to_string());

    process_message(&mut ctx.accounts.rollback_account, &signer, &to, &envelope)?;

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

    require_gte!(
        constants::MAX_DATA_SIZE,
        encode_msg.len() as usize,
        XcallError::MaxDataSizeExceeded
    );

    if is_reply(&ctx.accounts.reply, &to.nid(), &envelope.sources) && !need_response {
        ctx.accounts.reply.call_reply = Some(request.clone());
    } else {
        // TODO: call connection and claim protocol fee
    }

    Ok(sequence_no)
}

pub fn process_message(
    rollback_account: &mut Option<Account<RollbackAccount>>,
    from: &AccountInfo,
    to: &NetworkAddress,
    envelope: &Envelope,
) -> Result<()> {
    match &envelope.message {
        AnyMessage::CallMessage(_) => {
            if rollback_account.is_some() {
                return Err(XcallError::RollbackAccountShouldNotBeCreated.into());
            }
            Ok(())
        }
        AnyMessage::CallMessagePersisted(_) => {
            if rollback_account.is_some() {
                return Err(XcallError::RollbackAccountShouldNotBeCreated.into());
            }
            Ok(())
        }
        AnyMessage::CallMessageWithRollback(msg) => {
            if !from.executable {
                return Err(XcallError::RollbackNotPossible.into());
            }
            require_gte!(
                constants::MAX_ROLLBACK_SIZE,
                msg.rollback().unwrap().len(),
                XcallError::MaxRollbackSizeExceeded
            );
            if envelope.message.rollback().is_some() {
                let rollback_data = envelope.message.rollback().unwrap();
                let rollback = Rollback::new(
                    from.key(),
                    to.clone(),
                    envelope.sources.clone(),
                    rollback_data,
                    false,
                );

                if let Some(rollback_account) = rollback_account {
                    rollback_account.rollback = rollback
                } else {
                    return Err(XcallError::RollbackAccountNotSpecified.into());
                }
            }
            Ok(())
        }
    }
}

pub fn is_reply(reply: &Account<Reply>, nid: &NetId, sources: &Vec<String>) -> bool {
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
pub struct SendCallCtx<'info> {
    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
      mut,
      seeds = ["reply".as_bytes()],
      bump
    )]
    pub reply: Account<'info, Reply>,

    /// TODO: include sequence no in seeds
    #[account(
      init,
      payer = signer,
      space = 8 + 1024,
      seeds = [RollbackAccount::SEED_PREFIX.as_bytes()],
      bump,
    )]
    pub rollback_account: Option<Account<'info, RollbackAccount>>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
