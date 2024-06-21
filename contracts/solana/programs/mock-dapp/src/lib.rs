use anchor_lang::prelude::*;

declare_id!("JB2StC5wcWiapQ8f5X18Ef1gWnU19UKfRY9DLhNWk143");

#[program]
pub mod mock_dapp {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
