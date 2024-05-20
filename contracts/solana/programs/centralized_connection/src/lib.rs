use anchor_lang::prelude::*;

declare_id!("7WRpFWLhZ9cZDtuyGqVA93YQacydHKXirqY9cYZaDaZd");

#[program]
pub mod centralized_connection {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let connection_account = &mut ctx.accounts.connection_account;
        connection_account.admin = ctx.accounts.user.key();

        msg!(
            "connection public key: {}",
            connection_account.key().to_string()
        );
        Ok(())
    }

    pub fn set_fee(ctx: Context<SetFee>, network_id: String, send_fee: u64, response_fee: u64) -> Result<()> {
        let fee_account = &mut ctx.accounts.fee_account;
        require_keys_eq!(
            ctx.accounts.connection_account.admin.key(),
            ctx.accounts.admin.key()
        );
        fee_account.message_fees = send_fee;
        fee_account.response_fees = response_fee;
        Ok(())
    }

    // pub fn claim_fees() {
    // save 2 yrs worth of rent
    // send remaining to the admin
    // }

    pub fn send_message(
        ctx: Context<SendMessage>,
        to: String,
        sn: u64,
        msg: Vec<u8>,
    ) -> Result<()> {
        msg!(
            "Sending message to: {} with sn: {} and msg: {:?}",
            to,
            sn,
            msg
        );

        // asset for balances

        let conn_state = &mut ctx.accounts.connection_account;
        conn_state.conn_sn += 1;
        // Add logic to handle the message sending here
        emit!(MessageEvent {
            target_network: to,
            sn: conn_state.conn_sn,
            _msg: msg,
        });

        Ok(())
    }
}

#[event]
pub struct MessageEvent {
    pub target_network: String,
    pub sn: u64,
    pub _msg: Vec<u8>,
}

#[account]
#[derive(InitSpace)]
pub struct FeesState {
    message_fees: u64,
    response_fees: u64,
}

#[account]
pub struct ConnectionState {
    pub conn_sn: u64,
    pub admin: Pubkey,
}

#[derive(Accounts)]
#[instruction(network_id: String)]
pub struct SetFee<'info> {
    #[account(
        init, 
        seeds=[b"connection_fee", network_id.as_bytes()], 
        space = 8 + FeesState::INIT_SPACE, 
        payer = admin, 
        bump,
    )]
    pub fee_account: Account<'info, FeesState>,

    pub connection_account: Account<'info, ConnectionState>,
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, seeds=[b"connection_state"], payer = user, space = 8 + 32 + 16, bump)]
    pub connection_account: Account<'info, ConnectionState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendMessage<'info> {
    #[account(mut)]
    pub connection_account: Account<'info, ConnectionState>,

    pub fee_account: Account<'info, FeesState>,
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid fee calculation")]
    InvalidFeeCalculation,
}
