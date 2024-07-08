use anchor_lang::prelude::*;

use crate::types::{
    message::{CSMessage, CSMessageDecoded, CSMessageType},
    request::CSMessageRequest,
    result::CSMessageResult,
};

pub fn decode_cs_message(message: Vec<u8>) -> Result<CSMessageDecoded> {
    let cs_message: CSMessage = message.try_into()?;

    match cs_message.message_type() {
        CSMessageType::CSMessageRequest => {
            let request: CSMessageRequest = cs_message.payload().try_into()?;
            Ok(CSMessageDecoded {
                message_type: cs_message.message_type,
                request: Some(request),
                result: None,
            })
        }
        CSMessageType::CSMessageResult => {
            let result: CSMessageResult = cs_message.payload().try_into()?;
            Ok(CSMessageDecoded {
                message_type: cs_message.message_type,
                request: None,
                result: Some(result),
            })
        }
    }
}

#[derive(Accounts)]
pub struct EmptyContext {}
