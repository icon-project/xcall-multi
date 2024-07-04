use anchor_lang::prelude::*;

use crate::event::*;
use crate::types::{
    message::{CSMessage, CSMessageDecodedType, CSMessageType},
    request::CSMessageRequest,
    result::CSMessageResult,
};

pub fn decode_cs_message(message: Vec<u8>) -> Result<()> {
    let cs_message: CSMessage = message.try_into()?;

    match cs_message.message_type() {
        CSMessageType::CSMessageRequest => {
            let request: CSMessageRequest = cs_message.payload().try_into()?;
            emit!(CSMessageDecoded {
                msgType: cs_message.message_type,
                msg: CSMessageDecodedType::CSMessageRequest(request)
            })
        }
        CSMessageType::CSMessageResult => {
            let result: CSMessageResult = cs_message.payload().try_into()?;
            emit!(CSMessageDecoded {
                msgType: cs_message.message_type,
                msg: CSMessageDecodedType::CSMessageResult(result)
            })
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct EmptyContext {}
