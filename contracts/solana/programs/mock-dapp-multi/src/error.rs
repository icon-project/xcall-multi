use anchor_lang::prelude::error_code;

#[error_code]
pub enum DappError {
    #[msg("Address Mismatch")]
    AddressMismatch,

    #[msg("Rollback Mismatch")]
    RollbackMismatch,

    #[msg("Invalid Rollback Message")]
    InvalidRollbackMessage,

    #[msg("Uninitialized")]
    Uninitialized,

    #[msg("Invalid Source")]
    InvalidSource,

    #[msg("Only xcall")]
    OnlyXcall,
}
