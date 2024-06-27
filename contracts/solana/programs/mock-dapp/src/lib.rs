use anchor_lang::prelude::*;

declare_id!("Gqh6NKE7tRNMpZLGotxrUZA1m2ndnUGbGsyU1KQFs9wa");

#[program]
pub mod mock_dapp {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
