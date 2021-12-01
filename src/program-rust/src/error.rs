//! Error types
#![allow(dead_code)]

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the GovernanceTools
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum AccountError {
    /// Account already initialized
    #[error("Account already initialized")]
    AccountAlreadyInitialized = 1100,

    /// Account doesn't exist
    #[error("Account doesn't exist")]
    AccountDoesNotExist,

    /// Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner,

    /// Invalid Account type
    #[error("Invalid Account type")]
    InvalidAccountType,
}

impl PrintProgramError for AccountError {
    fn print<E>(&self) {
        msg!("ACCOUNT-ERROR: {}", &self.to_string());
    }
}

impl From<AccountError> for ProgramError {
    fn from(e: AccountError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for AccountError {
    fn type_of() -> &'static str {
        "Governance Tools Error"
    }
}
