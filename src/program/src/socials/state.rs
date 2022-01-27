use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

/// Used to prefix accounts
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]

pub enum AccountContainer {
    UserAccount(UserAccount),
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ProfilePicture {
    NFT { id: Pubkey }, // MetaPlex standard
    Unverified { url: String },
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct UserDescription {}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub name: String,
    pub profile: Option<ProfilePicture>,
    pub description: Option<UserDescription>,
}

pub fn deserialize_user_account(data: &[u8]) -> Result<UserAccount> {
    let AccountContainer::UserAccount(account) = try_from_slice_unchecked(data)?;
    return Ok(account);
}
