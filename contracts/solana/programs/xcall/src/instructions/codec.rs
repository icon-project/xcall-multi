use anchor_lang::prelude::*;

use crate::types::{
    message::{CSMessage, CSMessageDecoded, CSMessageDecodedType, CSMessageType},
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
                msg: CSMessageDecodedType::CSMessageRequest(request),
            })
        }
        CSMessageType::CSMessageResult => {
            let result: CSMessageResult = cs_message.payload().try_into()?;
            Ok(CSMessageDecoded {
                message_type: cs_message.message_type,
                msg: CSMessageDecodedType::CSMessageResult(result),
            })
        }
    }
}

#[derive(Accounts)]
pub struct EmptyContext {}
