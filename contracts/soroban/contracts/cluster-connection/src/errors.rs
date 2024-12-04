use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    OnlyAdmin = 1,
    Uninitialized = 2,
    AlreadyInitialized = 3,
    InsufficientFund = 4,
    DuplicateMessage = 5,
    NetworkNotSupported = 6,
    CannotRemoveAdmin = 7,
    ThresholdExceeded = 8,
    ValidatorNotFound = 9,
    ValidatorAlreadyAdded = 10,
    SignatureVerificationFailed = 11,
}
