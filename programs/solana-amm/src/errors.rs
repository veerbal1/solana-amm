use anchor_lang::prelude::*;
#[error_code]
pub enum AMMError {
    #[msg("Invalid Mint order")]
    InvalidMintOrder,

    #[msg("InsufficientFundsProvided")]
    InsufficientFundsProvided,
}
