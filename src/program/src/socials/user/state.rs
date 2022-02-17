use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use crate::{
    instruction::S2GAccountType,
    socials::{post::state::ContentSource, state::AccountType, MaxSize},
};

pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_USER_LEN: usize = 32 // owner pubkey
    + 8 // timestamp
    + MAX_NAME_LENGTH
    + 1  // option
    + MAX_URI_LENGTH
    + 200; // some padding

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct UserAccount {
    pub account_type: S2GAccountType,
    pub social_account_type: AccountType,
    pub owner: Pubkey,
    pub creation_timestamp: u64,
    pub name: String,
    pub profile: Option<ContentSource>, // The link to the profile data
}

impl MaxSize for UserAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_USER_LEN)
    }
}

pub fn deserialize_user_account(data: &[u8]) -> Result<UserAccount> {
    let user_account: UserAccount = try_from_slice_unchecked(data)?;
    return Ok(user_account);
}
