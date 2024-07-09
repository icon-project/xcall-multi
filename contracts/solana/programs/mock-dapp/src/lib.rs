use anchor_lang::prelude::*;

declare_id!("Fc87aXbgevB6LS2qXBYMrjdsoE1XSRF9ygTGLC2S1479");

#[program]
pub mod mock_dapp {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
