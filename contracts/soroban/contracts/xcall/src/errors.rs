use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractError {
    Uninitialized = 1,
    AlreadyInitialized = 2,
    NoDefaultConnection = 3,
    MaxRollbackSizeExceeded = 4,
    MaxDataSizeExceeded = 5,
    InsufficientFunds = 6,
    RollbackNotPossible = 7,
    RollbackNotEnabled = 8,
    ProtocolsMismatch = 9,
    InvalidRequestId = 10,
    DataMismatch = 11,
    MessageTypeNotSupported = 12,
    InvalidType = 13,
    CallRequestNotFound = 14,
    InvalidReplyReceived = 15,
    InvalidRlpLength = 16,
    NoRollbackData = 17,
}
