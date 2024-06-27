use std::{str::FromStr, vec};

use anchor_lang::{
    prelude::*, solana_program::{
        hash, instruction::Instruction, program::{get_return_data, invoke_signed},
    }
};

use xcall_lib::{message::msg_type::MessageType, 
    network_address::NetworkAddress, 
    state::CpiDappResponse};

use crate::{
    error::XcallError, event, state::RollbackAccount, DefaultConnection 
};

pub fn execute_rollback<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExecuteRollbackCtx<'info>>,
    _sn: u128,
) -> Result<()> {

    let req =  ctx.accounts.rollback.as_mut()
    .ok_or(XcallError::InvalidSn)?;

   
    if !req.rollback.enabled() {
        return Err(XcallError::RollbackNotEnabled.into());
    }
    
    let to = &req.rollback.to();
    let from = &req.rollback.from().to_string();
    let data = &req.rollback.rollback().to_vec();
    let protocols = req.rollback.protocols().to_vec();
    
    pub struct CSMessageRequest {
    from: NetworkAddress,
    to: String,
    sequence_no: u128,
    msg_type: MessageType,
    data: Vec<u8>, // TODO: cosmos this is nullable??
    protocols: Vec<String>,
}
    let signer =  &ctx.accounts.signer;
    let program = &ctx.accounts.system_program;
    let remaining_accounts = ctx.remaining_accounts;
    let signer_seeds:&[&[&[u8]]]=&[&[DefaultConnection::SEED_PREFIX.as_bytes(), &[ctx.accounts.default_connection.as_mut().unwrap().bump]]];

    handle_call_message(signer, 
        program, 
        remaining_accounts, 
        signer_seeds, 
        from, 
        to,
        data, 
        protocols)?;

   emit!(event::RollbackExecuted {
                sn: _sn,
              
            });

    Ok(())
}



#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DappArgs {
    pub from: String,
    pub data: Vec<u8>,
    pub protocols: Vec<String>,
}

pub fn handle_call_message<'info>(
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


#[derive(Accounts)]
#[instruction(_sn : u128,)]
pub struct ExecuteRollbackCtx<'info> {
    #[account(
        mut, 
        seeds = [b"rollback", &_sn.to_le_bytes()], 
        bump = rollback.bump , 
        close = owner
    )]
    pub rollback: Option<Account<'info, RollbackAccount>>,

    #[account(
        mut,
        seeds = ["conn".as_bytes()],
        bump
    )]
    pub default_connection : Option<Account<'info,DefaultConnection>>,

    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    
}