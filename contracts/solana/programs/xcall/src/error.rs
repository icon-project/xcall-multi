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

    #[msg("Rollback account is not specified")]
    RollbackAccountNotSpecified,

    #[msg("Rollback account creator not specified")]
    RollbackCreatorNotSpecified,

    #[msg("Pending request account is not specified")]
    PendingRequestAccountNotSpecified,

    #[msg("Pending request account creator is not specified")]
    PendingRequestCreatorNotSpecified,

    #[msg("Pending response account is not specified")]
    PendingResponseAccountNotSpecified,

    #[msg("Pending response account creator is not specified")]
    PendingResponseCreatorNotSpecified,

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

    #[msg("Invalid pubkey")]
    InvalidPubkey,

    #[msg("Invalid proxy request creator address")]
    InvalidProxyCreator,

    #[msg("Invalid response from dapp")]
    InvalidResponse,
}
