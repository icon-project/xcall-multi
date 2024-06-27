use anchor_lang::prelude::error_code;

#[error_code]
pub enum XcallError {
    #[msg("Only Admin")]
    OnlyAdmin,

    #[msg("Maximum rollback data size exceeded")]
    MaxRollbackSizeExceeded,

    #[msg("Invalid SN")]
    InvalidSn,

    #[msg("Rollback not enabled")]
    RollbackNotEnabled,
    
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

    #[msg("Invalid message seed")]
    InvalidMessageSeed,

    #[msg("Successful response account is not specified")]
    SuccessfulResponseAccountNotSpecified,

    #[msg("Protocol mismatch")]
    ProtocolMismatch,

    #[msg("Rollback not possible")]
    RollbackNotPossible,

    #[msg("Call request not found")]
    CallRequestNotFound,

    #[msg("No rollback data")]
    NoRollbackData,

    #[msg("Invalid reply received")]
    InvalidReplyReceived,

    #[msg("Invalid message sequence received")]
    InvalidMessageSequence,

    #[msg("Decode failed")]
    DecodeFailed,

    #[msg("Invalid source")]
    InvalidSource,

    #[msg("Invalid request id")]
    InvalidRequestId,

    #[msg("Data mismatch")]
    DataMismatch,
}
