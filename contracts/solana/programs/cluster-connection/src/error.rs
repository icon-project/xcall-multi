use anchor_lang::prelude::*;

#[error_code]
pub enum ConnectionError {
    #[msg("Only admin")]
    OnlyAdmin,

    #[msg("Only relayer")]
    OnlyRelayer,

    #[msg("Only xcall")]
    OnlyXcall,

    #[msg("Admin Validator Cnnot Be Removed")]
    AdminValidatorCnnotBeRemoved,

    #[msg("Validators Must Be Greater Than Threshold")]
    ValidatorsMustBeGreaterThanThreshold,
}
