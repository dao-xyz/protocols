use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked, msg, program_error::ProgramError, pubkey::Pubkey,
};

use super::MaxSize;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_USER_LEN: usize = 32 // owner pubkey
    + MAX_NAME_LENGTH
    + 1  // option
    + MAX_URI_LENGTH
    + 200; // some padding

/// Used to prefix accounts
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]

pub enum AccountContainer {
    UserAccount(UserAccount),
}

impl MaxSize for AccountContainer {
    fn get_max_size(&self) -> Option<usize> {
        match self {
            AccountContainer::UserAccount(user) => user.get_max_size(),
        }
    }
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub name: String,
    pub profile: Option<String>, // The link to the profile data
}

impl MaxSize for UserAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_USER_LEN)
    }
}

pub fn deserialize_user_account(data: &[u8]) -> Result<UserAccount> {
    let AccountContainer::UserAccount(account) = try_from_slice_unchecked(data)?;
    return Ok(account);
}
