use anchor_lang::prelude::*;

use crate::types::{
    message::{CSMessage, CSMessageType},
    request::CSMessageRequest,
    result::CSMessageResult,
};

pub fn decode_cs_message(
    message: Vec<u8>,
) -> Result<(
    CSMessageType,
    Option<CSMessageRequest>,
    Option<CSMessageResult>,
)> {
    let cs_message: CSMessage = message.try_into()?;

    match cs_message.message_type() {
        CSMessageType::CSMessageRequest => {
            let request: CSMessageRequest = cs_message.payload().try_into()?;
            Ok((CSMessageType::CSMessageRequest, Some(request), None))
        }
        CSMessageType::CSMessageResult => {
            let result: CSMessageResult = cs_message.payload().try_into()?;
            Ok((CSMessageType::CSMessageResult, None, Some(result)))
        }
    }
}

#[derive(Accounts)]
pub struct EmptyContext {}
