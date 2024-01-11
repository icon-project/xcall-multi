use super::*;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("DecodeError {error}")]
    DecodeError { error: String },
    #[error("RollBackMessageMismatch {sequence}")]
    RollBackMismatch { sequence: u64 },
    #[error("RevertFromDAPP")]
    RevertFromDAPP,
    #[error("ModuleAddressNotFound")]
    ModuleAddressNotFound,
    #[error("MisiingRollBack {sequence}")]
    MisiingRollBack { sequence: u64 },
    #[error("Connection Not Found {network_id}")]
    ConnectionNotFound { network_id: String },
    #[error("Invalid Address {address}")]
    InvalidAddress { address: String },
    #[error("Only Relayer(Admin)")]
    OnlyAdmin,
    #[error("Only XCall")]
    OnlyXCall,
    #[error("Duplicate Message")]
    DuplicateMessage,
    #[error("InsufficientFunds")]
    InsufficientFunds
}