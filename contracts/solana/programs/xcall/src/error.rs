use anchor_lang::prelude::error_code;

#[error_code]
pub enum XcallError {
    #[msg("Only Admin")]
    OnlyAdmin,

    #[msg("Maximum rollback data size exceeded")]
    MaxRollbackSizeExceeded,

    #[msg("Maximum data size exceeded")]
    MaxDataSizeExceeded,

    #[msg("Rollback account should not be created")]
    RollbackAccountShouldNotBeCreated,

    #[msg("Rollback account is not specified")]
    RollbackAccountNotSpecified,

    #[msg("Pending request account is not specified")]
    PendingRequestsAccountNotSpecified,

    #[msg("Pending request account is not specified")]
    PendingResponsesAccountNotSpecified,

    #[msg("Protocol mismatch")]
    ProtocolMismatch,

    #[msg("Rollback not possible")]
    RollbackNotPossible,

    #[msg("No rollback data")]
    NoRollbackData,

    #[msg("Decode failed")]
    DecodeFailed,
}
