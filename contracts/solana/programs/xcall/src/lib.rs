use anchor_lang::prelude::*;

pub mod types;

declare_id!("8zs31mXHopbEZ9RBJWXdFvPHZehnEMeSypkyVDjbTK5p");

#[program]
pub mod xcall {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
