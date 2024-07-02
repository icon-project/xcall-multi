use anchor_lang::prelude::error_code;

#[error_code]
pub enum NetworkError {
    #[msg("Invalid network address")]
    InvalidNetworkAddress,
}
