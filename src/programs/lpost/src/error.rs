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
pub enum PostError {
    /// Invalid owner for vote record
    #[error("Invalid owner for vote record")]
    InvalidOwnerForVoteRecord = 800,

    /// Invalid post for vote record
    #[error("Invalid post for vote record")]
    InvalidPostforVoteRecord,

    /// Vote already exist
    #[error("Vote already exist")]
    VoteAlreadyExist,
    /// Vote does not exist
    #[error("Vote does not exist")]
    VoteDoesNotExist,

    #[error("Post already exist")]
    PostAlreadyExist,

    /// Invaldi channel for post
    #[error("Invalid channel for post")]
    InvalidChannelForPost,

    /// Invalid tag for vote
    #[error("Invalid tag for vote")]
    InvalidTagForVote,

    /// Invalid tag authority
    #[error("Invalid tag authority")]
    InvaligTagAuthority,

    /// Invalid post for channel
    #[error("Invalid post for channel")]
    InvalidPostForChannel,
}

impl PrintProgramError for PostError {
    fn print<E>(&self) {
        msg!("POST-ERROR: {}", &self.to_string());
    }
}

impl From<PostError> for ProgramError {
    fn from(e: PostError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for PostError {
    fn type_of() -> &'static str {
        "Post Error"
    }
}
