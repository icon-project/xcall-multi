use anchor_lang::{
    prelude::*,
    solana_program::keccak,
};

use crate::{
    error::XcallError, event, state::RollbackAccount 
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
    
    let to = req.rollback.to();
    let protocols = req.rollback.protocols();
    let signer =  &ctx.accounts.signer;
    let program = &ctx.accounts.system_program;
    let remaining_accounts = ctx.remaining_accounts;
    let signer_seeds:&[&[&[u8]]]=&[&[DefaultConnection::SEED_PREFIX.as_bytes(), &[ctx.accounts.default_connection.bump]]];

    handle_call_message(signer, 
        program, 
        remaining_accounts, signer_seeds, 
        to,
        &req.rollback.from(), &data, protocols)?;

   emit!(event::RollbackExecuted {
                sn: _sn,
              
            });

    Ok(())
}


pub fn get_hash(data: &Vec<u8>) -> Vec<u8> {
    return keccak::hash(&data).as_ref().to_vec();
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
    pub default_connections : Option<Account<'info,DefaultConnection>>,

    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    
}