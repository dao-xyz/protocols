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
pub enum SocialError {
    #[error("Invalid owner for vote record")]
    InvalidOwnerForVoteRecord = 800,

    #[error("Invalid post for vote record")]
    InvalidPostforVoteRecord,

    #[error("Vote already exist")]
    VoteAlreadyExist,

    #[error("Vote does not exist")]
    VoteDoesNotExist,

    #[error("Post already exist")]
    PostAlreadyExist,

    /// Invaldi channel for post
    #[error("Invalid channel for post")]
    InvalidChannelForPost,

    #[error("Invalid tag for vote")]
    InvalidTagForVote,

    #[error("Invalid tag authority")]
    InvaligTagAuthority,

    #[error("Invalid post for channel")]
    InvalidPostForChannel,

    #[error("Invalid channel for authority")]
    InvalidChannelForAuthority,

    #[error("Invalid authority type")]
    InvalidAuthorityType,

    #[error("Invalid authority")]
    InvalidAuthority,

    #[error("Invalid parent channel")]
    InvalidParentChannel,

    #[error("Invalid channel type")]
    InvalidChannelType,
}

impl PrintProgramError for SocialError {
    fn print<E>(&self) {
        msg!("SOCIAL-ERROR: {}", &self.to_string());
    }
}

impl From<SocialError> for ProgramError {
    fn from(e: SocialError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SocialError {
    fn type_of() -> &'static str {
        "Post Error"
    }
}
