use anchor_lang::prelude::*;

#[error_code]
pub enum ConnectionError {
    #[msg("Only admin")]
    OnlyAdmin,

    #[msg("Only xcall")]
    OnlyXcall,
}
