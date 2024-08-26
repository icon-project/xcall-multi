use anchor_lang::prelude::error_code;

#[error_code]
pub enum XcallError {
    #[msg("Only Admin")]
    OnlyAdmin,

    #[msg("Invalid admin key")]
    InvalidAdminKey,

    #[msg("Invalid few handler")]
    InvalidFeeHandler,

    #[msg("Invalid signer")]
    InvalidSigner,

    #[msg("Maximum rollback data size exceeded")]
    MaxRollbackSizeExceeded,

    #[msg("Invalid SN")]
    InvalidSn,

    #[msg("Rollback not enabled")]
    RollbackNotEnabled,

    #[msg("Maximum data size exceeded")]
    MaxDataSizeExceeded,

    #[msg("Dapp authority not provided")]
    DappAuthorityNotProvided,

    #[msg("Protocol mismatch")]
    ProtocolMismatch,

    #[msg("Source protocols not specified")]
    SourceProtocolsNotSpecified,

    #[msg("Destination protocols not specified")]
    DestinationProtocolsNotSpecified,

    #[msg("Rollback not possible")]
    RollbackNotPossible,

    #[msg("Call request not found")]
    CallRequestNotFound,

    #[msg("No rollback data")]
    NoRollbackData,

    #[msg("Revert from dapp")]
    RevertFromDapp,

    #[msg("Invalid reply received")]
    InvalidReplyReceived,

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

    #[msg("Invalid response from dapp")]
    InvalidResponse,

    #[msg("Proxy request account is not specified")]
    ProxyRequestAccountNotSpecified,

    #[msg("Proxy request account must not be specified")]
    ProxyRequestAccountMustNotBeSpecified,

    #[msg("Rollback account is not specified")]
    RollbackAccountNotSpecified,

    #[msg("Rollback account must not be specified")]
    RollbackAccountMustNotBeSpecified,

    #[msg("Pending request account is not specified")]
    PendingRequestAccountNotSpecified,

    #[msg("Pending request account must not be specified")]
    PendingRequestAccountMustNotBeSpecified,

    #[msg("Pending response account is not specified")]
    PendingResponseAccountNotSpecified,

    #[msg("Pending response account must not be specified")]
    PendingResponseAccountMustNotBeSpecified,

    #[msg("Invalid message seed")]
    InvalidMessageSeed,

    #[msg("Successful response account is not specified")]
    SuccessfulResponseAccountNotSpecified,

    #[msg("Successful response account must not be specified")]
    SuccessfulResponseAccountMustNotBeSpecified,
}
