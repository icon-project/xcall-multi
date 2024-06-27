
use crate::execute_call::handle_call_message;

use anchor_lang::prelude::*;

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

#[derive(Accounts)]
#[instruction(_sn : u128,)]
pub struct ExecuteRollbackCtx<'info> {
    #[account(
        mut, 
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &_sn.to_string().as_bytes()], 
        bump = rollback.bump , 
        close = owner
    )]
    pub rollback: Option<Account<'info, RollbackAccount>>,

    #[account(
        mut,
        seeds = [DefaultConnection::SEED_PREFIX.as_bytes()],
        bump
    )]
    pub default_connection : Option<Account<'info,DefaultConnection>>,

    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    
}