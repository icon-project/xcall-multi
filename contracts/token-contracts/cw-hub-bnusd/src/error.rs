use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
    #[error("Wrong Address")]
    WrongAddress,
    #[error("Invalid Network Address according to Network ID")]
    InvalidNetworkAddress,
    #[error("Wrong Network")]
    WrongNetwork,
    #[error("Invalid to Address")]
    InvalidToAddress,
    #[error("OnlyCallService")]
    OnlyCallService,
    #[error("OnlyHub")]
    OnlyHub,
    #[error("Invalid Method")]
    InvalidMethod,
    #[error("Issue in Minting of Token")]
    MintError,
    #[error("Issue in Burning of Token")]
    BurnError,
    #[error("Invalid Data")]
    InvalidData,
    #[error("Address Not Found")]
    AddressNotFound,
    #[error("{0}")]
    Cw20BaseError(#[from] cw20_base::ContractError),
    #[error("Cannot Send to Self")]
    CannotSendToSelf,
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

// impl From<cw20_base::ContractError> for ContractError {
//     fn from(value: cw20_base::ContractError) -> Self {
//         todo!()
//     }
// }
