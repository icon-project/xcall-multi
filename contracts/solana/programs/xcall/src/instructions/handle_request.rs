use anchor_lang::prelude::*;

use crate::{CSMessageRequest, NetworkAddress, XCallState};

#[account]
pub struct ProxyReq {
    pub proxy_req: CSMessageRequest,
}

#[account]
pub struct RollbackDataAccount {
    pub from: Pubkey,
    pub to: String,
    pub protocols: Vec<String>,
    pub rollback: Vec<u8>,
    pub enabled: bool,
}

#[account] 
pub struct PendingResponses {
    pub result_sn: u128,
    pub sender: Vec<String>,
    pub status: Vec<bool>,

}

#[derive(Accounts)]
#[instruction(req_id: u128)]
pub struct HandleMessageCtx<'info> {
    #[account(init_if_needed, payer = user, space = 8 + 1024, seeds=[b"pending", req_id.to_le_bytes().as_ref() ], bump)]
    // #[account(init_if_needed, payer = user, space = 8 + 1024, seeds=[b"pending", user.key().as_ref(), req_id.to_le_bytes().as_ref() ], bump)]
    pub pending_responses: Account<'info, PendingResponses>,

    #[account(
        init_if_needed, 
        payer = user, space = 2048 ,seeds=[b"proxy_req", req_id.to_le_bytes().as_ref()],bump)]
    pub proxy_req: Account<'info, ProxyReq>,

    #[account(mut)]
    pub rollback_data: Account<'info, RollbackDataAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub xcall_state: Account<'info, XCallState>,

    /// CHECK: To handle fees
    #[account(mut)]
    pub fee_handler: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}


pub fn handle_request(ctx:Context<HandleMessageCtx>, from_nid: String, payload: Vec<u8>) -> Result<()> {
    let msg_req = CSMessageRequest::unmarshal_from(&payload).unwrap();
    let net = NetworkAddress::split(msg_req.from.clone()).net;
    assert_eq!(net, from_nid);

    // verify protocols

    let xcall_state = &mut ctx.accounts.xcall_state;
    let req_id = xcall_state.request_id + 1;
    xcall_state.request_id = req_id;

    let hashed_msg = msg_req.data_hash();

    let proxy_req = &mut ctx.accounts.proxy_req;
    proxy_req.proxy_req = hashed_msg;
    Ok(())
}
