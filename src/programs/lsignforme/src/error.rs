//! Error types
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Governance program
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum SignForMeError {
    #[error("Invalid owner")]
    InvalidOwner = 1300,

    #[error("Invalid signer")]
    InvalidSigner,

    #[error("SignForMe record already exist")]
    SignForMeRecordAlreadyExist,
}

impl PrintProgramError for SignForMeError {
    fn print<E>(&self) {
        msg!("SIGNFORME-ERROR: {}", &self.to_string());
    }
}

impl From<SignForMeError> for ProgramError {
    fn from(e: SignForMeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SignForMeError {
    fn type_of() -> &'static str {
        "SignForME Error"
    }
}
