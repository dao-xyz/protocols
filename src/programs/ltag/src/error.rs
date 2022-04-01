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
pub enum TagError {
    /// Invalid owner
    #[error("Invalid owner")]
    InvalidOwner = 500,

    /// Invalid authority
    #[error("Invalid authority")]
    InvalidAuthority,

    /// Invalid TagRecordFactory
    #[error("Invalid TagRecordFactory")]
    InvalidTagRecordFactory,

    /// Tag already exist
    #[error("Tag already exisst")]
    TagAlreadyExist,

    /// Tag record already exist
    #[error("Tag record already exist")]
    TagRecordAlreadyExist,

    /// TagRecordFactory already exist
    #[error("TagRecordFactory already exist")]
    TagRecordFactoryAlreadyExist,

    /// Tag does not exist
    #[error("Tag does not exist")]
    TagDoesNotExist,

    /// Invalid tag
    #[error("Invalid tag")]
    InvalidTag,
}

impl PrintProgramError for TagError {
    fn print<E>(&self) {
        msg!("TAG-ERROR: {}", &self.to_string());
    }
}

impl From<TagError> for ProgramError {
    fn from(e: TagError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TagError {
    fn type_of() -> &'static str {
        "Tag Error"
    }
}
