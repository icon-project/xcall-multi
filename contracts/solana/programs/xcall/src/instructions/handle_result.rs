use anchor_lang::prelude::*;

use crate::{
    CSMessageRequest, CSMessageResult, CallServiceResponseType, HandleMessageCtx, NetworkAddress,
};

pub fn handle_result(ctx: Context<HandleMessageCtx>, data: Vec<u8>) -> Result<()> {
    let result = CSMessageResult::unmarshal_from(&data).unwrap();
    let res_sn = result.sequence_no;
    let rollback_data = &mut ctx.accounts.rollback_data;

    // zero address check

    // pending checks

    match CallServiceResponseType::from(result.response_code) {
        CallServiceResponseType::CallServiceResponseFailure => {
            require_gte!(rollback_data.rollback.len(), 0);
            let new_rollback = &mut ctx.accounts.new_rollback_data;
            new_rollback.enabled = true;
            new_rollback.from = rollback_data.from;
            new_rollback.to = rollback_data.to.clone();
            new_rollback.protocols = rollback_data.protocols.clone();
            new_rollback.rollback = rollback_data.rollback.clone();

            emit!(RollbackMessage { sn: res_sn });
        }
        CallServiceResponseType::CallServiceResponseSuccess => {
            // delete account
            
            if result.message.len() > 0 {
                let mut reply = CSMessageRequest::unmarshal_from(&result.message).unwrap();
                let from_net = NetworkAddress::split(reply.from.clone()).net;
                require_eq!(rollback_data.to.clone(), from_net);
                reply.protocols = rollback_data.protocols.clone();

                let cloned_reply = reply.clone();

                let xcall_state = &mut ctx.accounts.xcall_state;
                let req_id = xcall_state.request_id + 1;
                xcall_state.request_id = req_id;

                emit!(CallMessage {
                    from: reply.from,
                    to: reply.to,
                    sn: reply.sequence_no,
                    req_id: req_id,
                    data: reply.data,
                });

                let hashed_reply = cloned_reply.data_hash();
                let proxy_req = &mut ctx.accounts.proxy_req;
                proxy_req.proxy_req = hashed_reply;
            }

            // save to successful responses
        }
    }

    Ok(())
}

#[event]
pub struct CallMessage {
    pub from: String,
    pub to: String,
    pub sn: u128,
    pub req_id: u128,
    pub data: Vec<u8>,
}

#[event]
pub struct RollbackMessage {
    pub sn: u128,
}
