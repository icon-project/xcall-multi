// test cases: invalid request id, unequal hash data, protocols with and without values
// each msg has different msg type


use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{
        hash, instruction::Instruction, keccak, program::{get_return_data, invoke_signed},
    },
};

use rlp::Encodable;
use xcall_lib::{message::msg_type::MessageType, 
    network_address::NetworkAddress, 
    state::CpiDappResponse};

use crate::{
    error::XcallError, event, state::ProxyRequest, 
    types::{message::CSMessage, result::{ CSMessageResult, CSResponseType}}, 
    DefaultConnection, Reply 
};

pub fn execute_call<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteCallCtx<'info>>,
    req_id: u128,
    data: Vec<u8>,
) -> Result<()> {

    let proxy_request = ctx.accounts.proxy_requests.as_mut()
        .ok_or(XcallError::InvalidRequestId)?;

    let request = &proxy_request.req; 

    
    if get_hash(&data) != request.data() {
        return Err(XcallError::DataMismatch.into());
    }

    // TODO: close proxy_requests account

    let to = request.to();
    let protocols = request.protocols();
    let signer =  &ctx.accounts.signer;
    let program = &ctx.accounts.system_program;
    let remaining_accounts = ctx.remaining_accounts;
    

    match proxy_request.req.msg_type() {
        MessageType::CallMessage => {
            try_handle_call_message(signer,
                program,
                remaining_accounts, 
                req_id, to, &request.from(), &data, protocols.clone())?;
        }
        MessageType::CallMessagePersisted => {
            handle_call_message(signer, 
                program, 
                remaining_accounts, &[&[b"xcall"]], 
                to,
                &request.from(), &data, protocols)?;
        }
        MessageType::CallMessageWithRollback => {

            if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
                rep.reply_state = Some(request.clone())
            }

            let dapp_response = handle_call_message(
                signer,
                program,
                remaining_accounts,
                &[&[b"xcall"]],
                to,
                &request.from(),
                &data,
                protocols.clone())?;

            // TODO:better way to set the reply state
            if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
                rep.reply_state = None
            }

            let sucess = dapp_response.success;
            let response_code;
            if sucess {
                response_code = CSResponseType::CSResponseSuccess
            }
            else {
                response_code = CSResponseType::CSResponseFailure
            }
            

            // TODO: from where the call reply is added and how to access it
            let mut message = vec![];
            if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
                if rep.call_reply.is_some() && sucess {
                    message = rep.call_reply.as_mut().unwrap().rlp_bytes().to_vec();
                }
            }

            // TODO:better way to set the call reply as none
            if let Some (rep) = ctx.accounts.reply_state.as_deref_mut() {
                rep.call_reply = None
            }

            let result = CSMessageResult::new(request.sequence_no(),
             response_code,Some(message));

            let cs_message = CSMessage::from(result);

            let nid = request.from().nid();
            let mut destinations = request.protocols().clone();

            if destinations.is_empty() {
                let default_connection = ctx.accounts.default_connections.as_deref_mut().unwrap();
                destinations = vec![default_connection.address.to_string()]
            }

            for to in destinations {
                // TODO: call connection contract --> the flow is not completed

                let ix_name = format!("{}:{}", "global", "send_message");
                let ix_discriminator = hash::hash(ix_name.as_bytes()).to_bytes()[..8].to_vec();

                let mut data = vec![];
                let args = SendMessageArgs {
                    to: nid.clone(),
                    sn: -(request.sequence_no() as i64),
                    msg: cs_message.as_bytes(),
                };
                args.serialize(&mut data)?;

                let mut ix_data = Vec::new();
                ix_data.extend_from_slice(&ix_discriminator);
                ix_data.extend_from_slice(&data);

                // TODO: the context of centralized call needs more
                let account_metas: Vec<AccountMeta> = vec![
                    AccountMeta::new(ctx.accounts.signer.key(), true),
                    AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                ];

                let ix = Instruction {
                    program_id: Pubkey::from_str(&to).unwrap(),
                    accounts: account_metas,
                    data: ix_data.clone(),
                };

                let account_infos: Vec<AccountInfo<'info>> = vec![
                    ctx.accounts.signer.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                  
                ];

                // TODO: figure out the signer seeds
                invoke_signed(&ix, &account_infos, &[&[b"execute+call"]])?

            }

        }
    }

    Ok(())
}
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendMessageArgs {
    pub to: String,
    pub sn: i64,
    pub msg: Vec<u8>,
}
// from and data
pub fn try_handle_call_message<'a, 'b, 'c, 'info>(
    signer : &Signer<'info>,
    program : &Program<'info,System>,
    remaining_accounts :&[AccountInfo<'info>],
    req_id: u128,
    to: &String,
    from: &NetworkAddress,
    data: &Vec<u8>,
    _protocols: Vec<String>,
) -> Result<()> {


    let mut protocols: Option<Vec<String>> = None;
    if _protocols.len() > 0 {
        protocols = Some(_protocols)
    }

    // todo: need to handle the unwrap here
    let dapp_response = handle_call_message(signer, 
        program, 
        remaining_accounts, &[&[b"xcall"]], 
        to, 
        from,
        &data, protocols.unwrap())?;

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
                msg: dapp_response.data.unwrap(),
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

// TODO: pass the xcall signer seed here
pub fn handle_call_message<'c,'info>(
    signer : &Signer<'info>,
    program : &Program<'info,System>,
    remaining_accounts :&[AccountInfo<'info>],
    signers_seeds: &[&[&[u8]]],
    to: &String,
    from: &NetworkAddress,
    _data : &Vec<u8>,
    protocols: Vec<String>

) -> Result<CpiDappResponse>{

    let ix_name = format!("{}:{}", "global", "handle_call_message");
    let ix_discriminator = hash::hash(ix_name.as_bytes()).to_bytes()[..8].to_vec();

    let mut data = vec![];
    let args = DappArgs {
        from: from.to_string(),
        data: _data.clone(),
        protocols: protocols
    };
    args.serialize(&mut data)?;

    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&ix_discriminator);
    ix_data.extend_from_slice(&data);

    let mut account_metas: Vec<AccountMeta> = vec![
        AccountMeta::new(signer.key(), true),
        AccountMeta::new_readonly(program.key(), false),

    ];

    let mut account_infos:Vec<AccountInfo<'info>> = vec![
        signer.to_account_info(),
        program.to_account_info(),
        ];

    for accounts in remaining_accounts {
        account_metas.push(AccountMeta::new(accounts.key(), accounts.is_signer));
        account_infos.push(accounts.to_account_info());
    }

    let ix = Instruction {
        program_id: Pubkey::from_str(to).unwrap(),
        accounts: account_metas,
        data: ix_data,
    };

    invoke_signed(&ix, &account_infos, signers_seeds)?;

    let (_, data) = get_return_data().unwrap();

    let mut data_slice : &[u8] = &data;
    let data_ref : &mut &[u8] = &mut data_slice;
    let deserialized = CpiDappResponse::deserialize(data_ref).unwrap();

    Ok(deserialized)

}

pub fn get_hash(data: &Vec<u8>) -> Vec<u8> {
    return keccak::hash(&data).as_ref().to_vec();
}

#[derive(Accounts)]
#[instruction(req_id : u128,)]
pub struct ExecuteCallCtx<'info> {
    #[account(
        mut, 
        seeds = ["proxy".as_bytes(), &req_id.to_le_bytes()], 
        bump = proxy_requests.bump)]
    pub proxy_requests: Option<Account<'info, ProxyRequest>>,

    #[account(
        mut,
        seeds = [Reply::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub reply_state: Option<Account<'info, Reply>>,

    #[account(
        mut,
        seeds = ["conn".as_bytes()],
        bump
    )]
    pub default_connections : Option<Account<'info,DefaultConnection>>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    
}
