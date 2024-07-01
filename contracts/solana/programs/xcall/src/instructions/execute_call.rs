use std::{str::FromStr, vec};

use anchor_lang::{
    prelude::*, solana_program::{
        hash, instruction::Instruction, program::{get_return_data, invoke_signed},
    }
};

use rlp::Encodable;
use xcall_lib::{message::msg_type::MessageType, 

    state::CpiDappResponse};

use crate::{
    error::XcallError, event, state::*, 
    types::{message::CSMessage, request:: CSMessageRequest, 
        result::{ CSMessageResult, CSResponseType}}
};

pub fn execute_call<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteCallCtx<'info>>,
    req_id: u128,
    data: Vec<u8>,
    from_nid: String
) -> Result<()> {
   
    let proxy_request = ctx.accounts.proxy_requests.as_mut().cloned()
    .ok_or(XcallError::InvalidRequestId)?;

    let request = &proxy_request.req; 

    if get_hash(&data) != request.data() {
        return Err(XcallError::DataMismatch.into());
    }

    // TODO: close proxy_requests account

    // let to = request.to();
    let protocols = request.protocols();
    
    

    match proxy_request.req.msg_type() {
        MessageType::CallMessage => {

            try_handle_call_message(ctx,
                req_id,
                request, 
                &data, protocols.clone())?;
        }
        MessageType::CallMessagePersisted => {
            
            handle_call_message(ctx,request,protocols, &data, false)?;
        }
        MessageType::CallMessageWithRollback => {

            if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
                rep.set_reply_state(Some(request.clone()))
            }

            handle_call_message(ctx,request,protocols,&data,true)?;
        }
    }

    Ok(())
}


pub fn try_handle_call_message<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteCallCtx<'info>>,
    req_id: u128,
    request: &CSMessageRequest,
    data: &Vec<u8>,
    _protocols: Vec<String>,
) -> Result<()> {


    let mut protocols: Option<Vec<String>> = None;
    if _protocols.len() > 0 {
        protocols = Some(_protocols)
    }

    // todo: need to handle the unwrap here
    let dapp_response = handle_call_message(ctx,
    request, protocols.unwrap_or_default(),&data, false)?;

    match dapp_response.success {
        true => {
            emit!(event::CallExecuted {
                reqId: req_id,
                code: CSResponseType::CSResponseSuccess.into(),
                msg: "success".to_string(),
            });
        }
        false => {
            emit!(event::CallExecuted {
                reqId: req_id,
                code: CSResponseType::CSResponseFailure.into(),
                msg: dapp_response.data.unwrap_or_default(),
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DappArgs {
    pub from: String,
    pub data: Vec<u8>,
    pub protocols: Vec<String>,
}


pub fn handle_call_message<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteCallCtx<'info>>,
    request : &CSMessageRequest,
    protocol: Vec<String>,
    data : &Vec<u8>,
    to_connection : bool

) -> Result<CpiDappResponse>{

    let ix_name = format!("{}:{}", "global", "handle_call_message");
    let ix_discriminator = hash::hash(ix_name.as_bytes()).to_bytes()[..8].to_vec();

    let mut dapp_args = vec![];
    let args = DappArgs {
        from: request.from().to_string(),
        data: data.clone(),
        protocols: protocol
    };
    args.serialize(&mut dapp_args)?;

    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&ix_discriminator);
    ix_data.extend_from_slice(&data);

    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(ctx.accounts.signer.key(), true),
        AccountMeta::new(ctx.accounts.system_program.key(),false)

    ];

    let mut account_infos:Vec<AccountInfo<'info>> = vec![
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ];

    for accounts in ctx.remaining_accounts {
        account_metas.push(AccountMeta::new(accounts.key(), accounts.is_signer));
        account_infos.push(accounts.to_account_info());
    }

    let ix = Instruction {
        program_id:Pubkey::from_str(request.to()).map_err(|_| XcallError::InvalidPubkey)?,
        accounts: account_metas,
        data: ix_data,
    };
    let signer_seeds:&[&[&[u8]]] =&[&[DefaultConnection::SEED_PREFIX.as_bytes(), &[ctx.accounts.default_connection.bump]]];
    
    invoke_signed(&ix, &account_infos, signer_seeds)?;

    let (_, data) = get_return_data().unwrap();
    

    let mut data_slice : &[u8] = &data;
    let data_ref : &mut &[u8] = &mut data_slice;
    let deserialized = CpiDappResponse::deserialize(data_ref).unwrap();


    if to_connection {
        call_connection(ctx,request, &data, &deserialized)?
    }

    Ok(deserialized)

}

pub fn call_connection<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteCallCtx<'info>>,
    request : &CSMessageRequest,
    data : &Vec<u8>,
    dapp_response: &CpiDappResponse) -> Result<()>{

    if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
        rep.set_reply_state(None)
    }

    let sucess = dapp_response.success;
    let response_code;
    if sucess {
        response_code = CSResponseType::CSResponseSuccess
    }
    else {
        response_code = CSResponseType::CSResponseFailure
    }
    
    let mut message = vec![];
    if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
        if rep.call_reply.is_some() && sucess {
            message = rep.call_reply.as_mut().unwrap().rlp_bytes().to_vec();
        }
    }

    if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
        rep.set_call_reply(None)
    }

    let result = CSMessageResult::new(request.sequence_no(),
     response_code,Some(message));

    let cs_message = CSMessage::from(result);

    let nid = request.from().nid();
    let mut destinations = request.protocols().clone();

    if destinations.is_empty() {
        let default_connection = ctx.accounts.default_connection.key();
        destinations = vec![default_connection.to_string()]
    }

    for (i,to) in destinations.iter().enumerate(){

        // TODO: should i check to with connection contract address -> if yes pass connection from remaining 
        let config = &ctx.remaining_accounts[4*i];
        let network_fee = &ctx.remaining_accounts[4*i+1];
        let claim_fee = &ctx.remaining_accounts[4*i+2];

        let ix_name = format!("{}:{}", "global", "send_message");
        let ix_discriminator = hash::hash(ix_name.as_bytes()).to_bytes()[..8].to_vec();

        let mut send_message_args = vec![];
        let args = SendMessageArgs {
            to: nid.clone(),
            sn: -(request.sequence_no() as i64),
            msg: cs_message.as_bytes(),
        };
        args.serialize(&mut send_message_args)?;

        let mut ix_data = Vec::new();
        ix_data.extend_from_slice(&ix_discriminator);
        ix_data.extend_from_slice(&data);

        let account_metas: Vec<AccountMeta> = vec![
            AccountMeta::new_readonly(ctx.accounts.default_connection.key(), true),
            AccountMeta::new(ctx.accounts.signer.key(), true),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
            AccountMeta::new(config.key(), false),
            AccountMeta::new_readonly(network_fee.key(), false),
            AccountMeta::new(claim_fee.key(), false),
        ];

        let ix = Instruction {
            program_id: Pubkey::from_str(&to).unwrap(),
            accounts: account_metas,
            data: ix_data.clone(),
        };

        let account_infos: Vec<AccountInfo<'info>> = vec![
            ctx.accounts.default_connection.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            config.to_account_info(),
            network_fee.to_account_info(),
            claim_fee.to_account_info(),
          
        ];

        let signer_seeds:&[&[&[u8]]] =&[&[DefaultConnection::SEED_PREFIX.as_bytes(), &[ctx.accounts.default_connection.bump]]];
        invoke_signed(&ix, &account_infos, signer_seeds)?

    

    }

    Ok(())

    
}

pub fn get_hash(data: &Vec<u8>) -> Vec<u8> {
    return hash::hash(data).to_bytes().to_vec();
}

#[derive(Accounts)]
#[instruction(req_id : u128, data:Vec<u8>,from_nid: String)]
pub struct ExecuteCallCtx<'info> {
    #[account(
        mut, 
        seeds = [ProxyRequest::SEED_PREFIX.as_bytes(), &req_id.to_string().as_bytes()], 
        bump = proxy_requests.bump)]
    pub proxy_requests: Option<Account<'info, ProxyRequest>>,

    #[account(
        mut,
        seeds = [Reply::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub reply_state: Option<Account<'info, Reply>>,

    #[account(
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes(), from_nid.as_bytes()],
        bump = default_connection.bump
    )]
    pub default_connection: Account<'info, DefaultConnection>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    
}
