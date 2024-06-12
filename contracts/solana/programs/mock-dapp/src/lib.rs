use anchor_lang::prelude::*;

declare_id!("8Q4FvsHCWK68EzYtsstdFYwUL1SHCiuLPRDJk1gaKiQ8");

#[program]
pub mod mock_dapp {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
