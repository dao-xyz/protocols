use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use crate::socials::{state::AccountContainer, MaxSize};
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_CHANNEL_LEN: usize = 32 // owner pubkey
    + MAX_NAME_LENGTH
    + 1  // option
    + MAX_URI_LENGTH
    + 200; // some padding

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ChannelAccount {
    pub owner: Pubkey,
    pub name: String,
    pub link: Option<String>, // The link to to info data
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_CHANNEL_LEN)
    }
}

impl ChannelAccount {
    pub fn new(owner: Pubkey, name: String, link: Option<String>) -> ChannelAccount {
        ChannelAccount { owner, name, link }
    }
}

pub fn deserialize_channel_account(data: &[u8]) -> Result<ChannelAccount> {
    if let AccountContainer::ChannelAccount(account) = try_from_slice_unchecked(data)? {
        return Ok(account);
    }
    panic!("Unkown data");
}
