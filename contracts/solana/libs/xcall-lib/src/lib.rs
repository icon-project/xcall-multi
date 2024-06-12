use anchor_lang::prelude::*;
pub mod message;
pub mod message_types;
pub mod network_address;

declare_id!("2XPhYhxGyoDx7B4EuGCGX2iBHfhTr352LEraKWDZbLr2");

#[program]
pub mod xcall_lib {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
