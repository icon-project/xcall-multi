use anchor_lang::{prelude::*, solana_program::hash};

use xcall_lib::message::call_message::CallMessage;
use xcall_lib::message::call_message_persisted::CallMessagePersisted;
use xcall_lib::message::call_message_rollback::CallMessageWithRollback;
use xcall_lib::message::{msg_type::*, AnyMessage};

use crate::{CallMessageCtx, DappError};

pub fn process_message(message_type: u8, data: Vec<u8>, rollback: Vec<u8>) -> Result<AnyMessage> {
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

pub fn get_network_connections(
    ctx: &Context<CallMessageCtx>,
) -> Result<(Vec<String>, Vec<String>)> {
    let connections = ctx.accounts.connections_account.connections.clone();

    let mut sources = Vec::new();
    let mut destinations = Vec::new();
    for conn in connections {
        sources.push(conn.src_endpoint);
        destinations.push(conn.dst_endpoint);
    }

    Ok((sources, destinations))
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
