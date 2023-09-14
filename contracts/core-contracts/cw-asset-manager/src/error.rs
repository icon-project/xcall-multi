use cosmwasm_std::{Addr, StdError};
use cw_ibc_rlp_lib::rlp::DecoderError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    //If a StdError is encountered and returned, it will be automatically converted into a ContractError using the #[from] attribute
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unacceptable token address: {address}")]
    InvalidToken { address: Addr },

    #[error("Sub call failed: {error}")]
    SubCallFailed { error: String },

    #[error("Token Deposit  Failed : {reason}")]
    DepositFailure { reason: String },

    #[error("Token Transfer Failed : {reason}")]
    TokenTransferFailure { reason: String },

    #[error("Deposit Revert Due to Xcall Failure : {account} : {token}")]
    RevertedDeposit { account: String, token: String },

    #[error("Xcall BTP Address is not found")]
    XAddressNotFound,

    #[error("unknown method extracted while decoding rlp bytes")]
    UnknownMethod,

    #[error("Insufficient token balance")]
    InsufficientTokenBalance,

    #[error("only xcall service is allowed")]
    OnlyXcallService,

    #[error("error in n/w check for xcall")]
    FailedXcallNetworkMatch,

    #[error("only contract owner is allowed")]
    OnlyOwner,

    #[error("only Icon Asset Manager is allowed")]
    OnlyIconAssetManager,

    #[error("xcall received data doesn't contained expected methods")]
    UnknownXcallDataReceived,

    #[error("invalid network address format for icon asset manager")]
    InvalidNetworkAddressFormat,

    #[error("invalid token address for cw20")]
    InvalidTokenAddress,

    #[error("Token amount can't be zero")]
    InvalidAmount,

    #[error("Recipient address is not proper network address")]
    InvalidRecipientAddress,

    #[error("Insufficient token allowance: CW20")]
    InsufficientTokenAllowance,

    #[error("Rlp Error: {error}")]
    DecoderError { error: DecoderError },
}

impl From<DecoderError> for ContractError {
    fn from(err: DecoderError) -> Self {
        ContractError::DecoderError { error: err }
    }
}
