use anchor_lang::prelude::*;

declare_id!("9hrkJ12DRZrbdQBwAGwgdjnauRj7Wue5stq1UzUyCJA3");

#[program]
pub mod mock_dapp {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
