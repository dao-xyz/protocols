use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the GovernanceTools
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum UtilsError {
    /// Account already initialized
    #[error("Account already initialized")]
    AccountAlreadyInitialized = 1100,

    /// Account doesn't exist
    #[error("Account doesn't exist")]
    AccountDoesNotExist, // 1101

    /// Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner,

    /// Invalid Account type
    #[error("Invalid Account type")]
    InvalidAccountType,
}

impl PrintProgramError for UtilsError {
    fn print<E>(&self) {
        msg!("UTILS-ERROR: {}", &self.to_string());
    }
}

impl From<UtilsError> for ProgramError {
    fn from(e: UtilsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for UtilsError {
    fn type_of() -> &'static str {
        "Utils Error"
    }
}
