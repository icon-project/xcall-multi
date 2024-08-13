use anchor_lang::prelude::*;

pub mod connection;
pub mod constants;
pub mod dapp;
pub mod error;
pub mod event;
pub mod helper;
pub mod instructions;
pub mod state;
pub mod types;

use instructions::*;

use types::message::CSMessageDecoded;
use xcall_lib::{
    network_address::NetworkAddress,
    query_account_type::{QueryAccountsPaginateResponse, QueryAccountsResponse},
};

declare_id!("47QmEHEPSQqhpEjok5PmooeqdqBXRVpU11aRMhJGe6LW");

#[program]
pub mod xcall {
    use super::*;

    /// Instruction: Initialize
    ///
    /// Initializes the initial program configuration
    ///
    /// This function sets up the initial configuration for the program, including specifying
    /// the network ID.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    /// - `network_id`: A string representing the network ID to be set in the configuration.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the initialization is successful, otherwise returns an
    /// error.
    pub fn initialize(ctx: Context<ConfigCtx>, network_id: String) -> Result<()> {
        instructions::initialize(ctx, network_id)
    }

    /// Instruction: Set Admin
    ///
    /// Sets a new admin account in the configuration.
    ///
    /// This function updates the admin account in the program’s configuration. Only the current
    /// admin (as verified by the context) can change the admin account.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    /// - `account`: The public key of the new admin account to be set.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the admin account is successfully updated, otherwise
    /// returns an error.
    pub fn set_admin(ctx: Context<SetAdminCtx>, account: Pubkey) -> Result<()> {
        instructions::set_admin(ctx, account)
    }

    /// Instruction: Set Protocol Fee
    ///
    /// Sets the protocol fee in the configuration account.
    ///
    /// This function verifies that the signer is an admin, and updates the protocol fee in the
    /// program's configuration account. The protocol fee is the amount charged for each
    /// cross-chain message sent.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    /// - `fee`: The new protocol fee to be set, specified as a `u64` value.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the protocol fee is successfully set, otherwise returns
    /// an error.
    pub fn set_protocol_fee(ctx: Context<SetFeeCtx>, fee: u64) -> Result<()> {
        instructions::set_protocol_fee(ctx, fee)
    }

    /// Instruction: Set Protocol Fee Handler
    ///
    /// Sets the specified pubkey as a protocol fee handler
    ///
    /// This function verifies that the signer is an admin of the program and sets `fee_handler` as
    /// a protocol fee handler. Typically, this is a designated fee collector or treasury account
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    /// - `fee_handler`: The pubkey of the new fee handler.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the transaction is successful, or an error if it fails.
    pub fn set_protocol_fee_handler(
        ctx: Context<SetFeeHandlerCtx>,
        fee_handler: Pubkey,
    ) -> Result<()> {
        instructions::set_protocol_fee_handler(ctx, fee_handler)
    }

    /// Instruction: Send Call
    ///
    /// Sends a cross-chain message to a specified network address.
    ///
    /// This function handles encoding, validation, and sending of a cross-chain message.
    /// It also manages the creation of a rollback account if needed and emits an event upon successful
    /// sending
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    /// - `message`: The `Envelope` payload, encoded as rlp bytes
    /// - `to`: The target network address where the message is to be sent
    ///
    /// # Returns
    /// - `Result<u128>`: The sequence number of the message if successful, wrapped in a `Result`.
    pub fn send_call<'info>(
        ctx: Context<'_, '_, '_, 'info, SendCallCtx<'info>>,
        envelope: Vec<u8>,
        to: NetworkAddress,
    ) -> Result<u128> {
        instructions::send_call(ctx, envelope, to)
    }

    #[allow(unused_variables)]
    pub fn handle_message(
        ctx: Context<HandleMessageCtx>,
        from_nid: String,
        msg: Vec<u8>,
        sequence_no: u128,
    ) -> Result<()> {
        instructions::handle_message(ctx, from_nid, msg)
    }

    pub fn handle_error<'info>(
        ctx: Context<'_, '_, '_, 'info, HandleErrorCtx<'info>>,
        sequence_no: u128,
    ) -> Result<()> {
        instructions::handle_error(ctx, sequence_no)
    }

    /// Instruction: Get Fee
    ///
    /// Calculates and retrieves the total fee for a cross-chain message, including the protocol fee
    /// and connection-specific fees.
    ///
    /// This function computes the total fee required to send a cross-chain message by adding the
    /// protocol fee stored in the configuration account and any additional fees specific to the
    /// connections used in the message.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction.
    /// - `nid`: A string representing the network ID for which the fee is being calculated.
    /// - `is_rollback`: A boolean indicating whether a rollback is required, affecting the fee.
    /// - `sources`: A vector of strings representing the source protocols involved in the transaction.
    ///
    /// # Returns
    /// - `Result<u64>`: Returns the total fee as a `u64` value if successful, otherwise returns
    /// an error.
    pub fn get_fee(
        ctx: Context<GetFeeCtx>,
        nid: String,
        is_rollback: bool,
        sources: Option<Vec<String>>,
    ) -> Result<u64> {
        instructions::get_fee(ctx, nid, is_rollback, sources.unwrap_or(vec![]))
    }

    /// Instruction: Get Admin
    ///
    /// Retrieves the admin public key from the configuration.
    ///
    /// This function returns the public key of the admin account, as stored in the configuration
    /// account.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    ///
    /// # Returns
    /// - `Result<Pubkey>`: Returns the public key of the admin account if successful,
    ///   otherwise returns an error.
    pub fn get_admin(ctx: Context<GetConfigCtx>) -> Result<Pubkey> {
        Ok(ctx.accounts.config.admin)
    }

    /// Instruction: Get Protocol Fee
    ///
    /// Retrieves the current protocol fee from the configuration.
    ///
    /// This function returns the protocol fee amount stored in the configuration account.
    /// The protocol fee is a value used to determine the amount charged for each cross-chain
    /// message.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    ///
    /// # Returns
    /// - `Result<u64>`: Returns the protocol fee as a `u64` value if successful,
    ///   otherwise returns an error.
    pub fn get_protocol_fee(ctx: Context<GetConfigCtx>) -> Result<u64> {
        Ok(ctx.accounts.config.protocol_fee)
    }

    /// Instruction: Get Protocol Fee Handler
    ///
    /// Retrieves the protocol fee handler public key from the configuration.
    ///
    /// This function returns the public key of the protocol fee handler account, as stored
    /// in the configuration account.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    ///
    /// # Returns
    /// - `Result<Pubkey>`: Returns the public key of the fee handler account if successful,
    ///   otherwise returns an error.
    pub fn get_protocol_fee_handler(ctx: Context<GetConfigCtx>) -> Result<Pubkey> {
        Ok(ctx.accounts.config.fee_handler)
    }

    /// Instruction: Get Network Address
    ///
    /// Retrieves the network address from the configuration.
    ///
    /// This function constructs and returns a `NetworkAddress` based on the network ID stored
    /// in the configuration account and the program's ID.
    ///
    /// # Arguments
    /// - `ctx`: The context of the solana program instruction
    ///
    /// # Returns
    /// - `Result<NetworkAddress>`: Returns the constructed `NetworkAddress` if successful,
    ///   otherwise returns an error.
    pub fn get_network_address(ctx: Context<GetConfigCtx>) -> Result<NetworkAddress> {
        Ok(NetworkAddress::new(
            &ctx.accounts.config.network_id,
            &id().to_string(),
        ))
    }

    /// Instruction: Decode CS Message
    ///
    /// Decodes a cross-chain message into its constituent parts.
    ///
    /// This function takes a serialized cross-chain message (`CSMessage`) and decodes it into
    /// a structured format (`CSMessageDecoded`). Depending on the message type, it will decode
    /// the message as either a `CSMessageRequest` or `CSMessageResult`. The decoded message
    /// is returned as a `CSMessageDecoded` struct, which contains either the request or the result.
    ///
    /// # Parameters
    /// - `ctx`: The context of the solana program instruction
    /// - `message`: A vector of bytes representing the serialized cross-chain message to be decoded
    ///
    /// # Returns
    /// - `Result<CSMessageDecoded>`: Returns the decoded message as a `CSMessageDecoded` struct
    /// if successful, otherwise returns an error.
    #[allow(unused_variables)]
    pub fn decode_cs_message<'info>(
        ctx: Context<'_, '_, '_, 'info, DecodeCSMessageContext<'info>>,
        message: Vec<u8>,
    ) -> Result<CSMessageDecoded> {
        instructions::decode_cs_message(message)
    }

    #[allow(unused_variables)]
    pub fn execute_call<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteCallCtx<'info>>,
        req_id: u128,
        data: Vec<u8>,
        from_nid: String,
    ) -> Result<()> {
        instructions::execute_call(ctx, req_id, data)
    }

    pub fn execute_rollback<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteRollbackCtx<'info>>,
        sn: u128,
    ) -> Result<()> {
        instructions::execute_rollback(ctx, sn)
    }

    pub fn query_execute_call_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryExecuteCallAccountsCtx<'info>>,
        req_id: u128,
        data: Vec<u8>,
        page: u8,
        limit: u8,
    ) -> Result<QueryAccountsPaginateResponse> {
        instructions::query_execute_call_accounts(ctx, req_id, data, page, limit)
    }

    #[allow(unused_variables)]
    pub fn query_execute_rollback_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryExecuteRollbackAccountsCtx<'info>>,
        sn: u128,
        page: u8,
        limit: u8,
    ) -> Result<QueryAccountsPaginateResponse> {
        instructions::query_execute_rollback_accounts(ctx, page, limit)
    }

    #[allow(unused_variables)]
    pub fn query_handle_message_accounts(
        ctx: Context<QueryHandleMessageAccountsCtx>,
        from_nid: String,
        msg: Vec<u8>,
        sequence_no: u128,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_handle_message_accounts(ctx, msg)
    }

    pub fn query_handle_error_accounts(
        ctx: Context<QueryHandleErrorAccountsCtx>,
        sequence_no: u128,
    ) -> Result<QueryAccountsResponse> {
        instructions::query_handle_error_accounts(ctx, sequence_no)
    }
}