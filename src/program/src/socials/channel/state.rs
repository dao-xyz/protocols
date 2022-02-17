use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use crate::{
    instruction::S2GAccountType,
    socials::{post::state::ContentSource, state::AccountType, MaxSize},
};
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_CHANNEL_LEN: usize = 32 // owner pubkey
    + 8 // timestamp
    + MAX_NAME_LENGTH
    + 1  // option
    + MAX_URI_LENGTH
    + 200; // some padding

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ChannelAccount {
    pub account_type: S2GAccountType,
    pub social_account_type: AccountType,
    pub owner: Pubkey,
    pub governence_mint: Pubkey,
    pub creation_timestamp: u64,
    pub name: String,
    pub link: Option<ContentSource>, // The link to to info data
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_CHANNEL_LEN)
    }
}

pub fn deserialize_channel_account(data: &[u8]) -> Result<ChannelAccount> {
    let account: ChannelAccount = try_from_slice_unchecked(data)?;
    return Ok(account);
}
