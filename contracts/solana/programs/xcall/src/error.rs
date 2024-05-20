use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Use send_message_with_rollback")]
    UseSendMessageWithRollback,

    #[msg("Use send_message")]
    UseSendMessage,

    #[msg("Size Exceed")]
    SizeExceed,

    #[msg("Invalid sources length")]
    InvalidSourcesLength,

    #[msg("Keys mismatch")]
    InvalidKeys,

    #[msg("Invalid Type")]
    InvalidType,

    #[msg("Insufficent Fee")]
    InsufficientFee,

    #[msg("Data Hash Mismatch")]
    DataHashMismatch,

    #[msg("XCallEnvelope RLP Decode Error")]
    XCallEnvelopeDecodeError,

    #[msg("CSMessageWithRollback RLP Decode Error")]
    CSMessageWithRollbackDecodeError,

    #[msg("CSMessageRequest RLP Decode Error")]
    CSMessageRequestDecodeError,
}
