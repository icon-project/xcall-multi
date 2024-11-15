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
    OrderNotFound = 7,
    OrderAlreadyFilled = 8,
    OrderMismatched = 9,
    InvalidEmitterAddress = 10,
    InvalidNetwork = 11,
    NetworkIdMisconfigured = 12,
}
