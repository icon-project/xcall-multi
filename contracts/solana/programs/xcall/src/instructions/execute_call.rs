use anchor_lang::{prelude::*, solana_program::keccak};

use crate::{error::ErrorCode, CSMessageRequest, MessageType, ProxyReq, ReplyData};

#[derive(Accounts)]
pub struct ExecuteCall<'info> {
    #[account(mut, close=fee_handler)]
    pub proxy_req: Account<'info, ProxyReq>,

    #[account(mut)]
    pub reply_data: Account<'info, ReplyData>,

    #[account(mut)]
    /// CHECK: Maybe needed
    pub fee_handler: AccountInfo<'info>,
}

pub fn execute_call(ctx: Context<ExecuteCall>, req_id: u128, data: Vec<u8>) -> Result<()> {
    let req = &mut ctx.accounts.proxy_req;
    // let proxy_request = req.proxy_req;

    // require_neq!(
    //     proxy_request.sequence_no,
    //     CSMessageRequest::null().sequence_no
    // );

    // // account proxy req gets closed in this function

    // let hash = keccak::hash(&data).as_ref().to_vec();
    // assert_eq!(hash, proxy_request.data);

    // match MessageType::from_int(proxy_request.msg_type) {
    //     MessageType::CallMessageNormal => {},
    //     MessageType::CallMessageWithRollback => {
    //         let reply_info = &mut ctx.accounts.reply_data;
    //         reply_info.reply_state = CSMessageRequest::null();

    //     },
    //     MessageType::CallMessagePersisted => {},
    // }
    Ok(())
}

pub fn execute_call_result() -> Result<()> {
    // emit!(CallExecuted{ red_id: todo!(), code: todo!(), msg: todo!() });
    Ok(())
}


#[event]
pub struct CallExecuted {
    pub red_id : u128, 
    pub code : u128,
    pub msg: String,
}