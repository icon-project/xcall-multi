use anchor_lang::prelude::*;
use xcall_lib::network_address::NetworkAddress;

use crate::{dapp, error::XcallError, event, id, state::*};

/// Executes a rollback operation using the stored rollback data.
///
/// This function retrieves the rollback data from the rollback account and prepares the necessary
/// instruction data to invoke the rollback operation in a DApp. It then executes the rollback
/// by calling the DApp's handle call message instruction, and emits an event if the rollback
/// is successful.
///
/// # Arguments
/// - `ctx`: The context containing all the necessary accounts and program state.
/// - `sn`: The sequence number associated with the rollback operation.
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the rollback was executed successfully, or an error if it
/// failed.
pub fn execute_rollback<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteRollbackCtx<'info>>,
    sn: u128,
) -> Result<()> {
    let rollback = &ctx.accounts.rollback_account.rollback;
    if !rollback.enabled() {
        return Err(XcallError::RollbackNotEnabled.into());
    }

    // Prepare the instruction data needed to invoke the rollback operation in the DApp.
    let ix_data = dapp::get_handle_call_message_ix_data(
        NetworkAddress::new(&ctx.accounts.config.network_id, &id().to_string()),
        rollback.rollback().to_owned(),
        rollback.protocols().clone(),
    )?;

    // Invoke the DApp's handle call message instruction with the prepared data and accounts.
    let res = dapp::invoke_handle_call_message_ix(
        rollback.from().to_owned(),
        ix_data,
        &ctx.accounts.config,
        &ctx.accounts.signer,
        &ctx.remaining_accounts,
    )?;

    // If the DApp reports a failure, log the error and return an error.
    if !res.success {
        msg!("Error executing rollback from dapp: {:?}", res.message);
        return Err(XcallError::RevertFromDapp.into());
    }

    emit!(event::RollbackExecuted { sn });

    Ok(())
}

#[derive(Accounts)]
#[instruction(sn : u128,)]
pub struct ExecuteRollbackCtx<'info> {
    /// The account that signs and pays for the transaction. This account is mutable
    /// because it will be debited for any fees or rent required during the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The solana system program account, used for creating and managing accounts.
    pub system_program: Program<'info, System>,

    /// The configuration account, which stores important settings and counters for the program.
    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is safe because this account is checked against the `config.admin` to ensure
    /// it is valid.
    #[account(
        mut,
        address = config.admin @ XcallError::OnlyAdmin
    )]
    pub admin: AccountInfo<'info>,

    /// The rollback account, identified by a sequence number (`sn`), used for executing rollback.
    /// The account is closed after use, with any remaining funds sent to the `admin`.
    #[account(
        mut,
        seeds = [RollbackAccount::SEED_PREFIX.as_bytes(), &sn.to_be_bytes()],
        bump = rollback_account.bump,
        close = admin,
    )]
    pub rollback_account: Account<'info, RollbackAccount>,
}
