use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use crate::socials::{state::AccountContainer, MaxSize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ContentSource {
    External { url: String }, // like ipfs
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Content {
    pub hash: [u8; 32],
    pub source: ContentSource,
}

pub const MAX_CONTENT_LEN: usize = 32 // hash pubkey
    + 200; // IPFS link (and some padding)

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Asset {
    NonAsset, // Not for sale, just a regular "post" (no one would want to buy this)
              // Add more markets here, like auction, then this would describe the owner token etc
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostAccount {
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub content: Content,
    pub asset: Asset,
}
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_POST_LEN: usize = 32 // creator pubkey
    + 32 // creator pubkey
    + 8 // timestamp
    + MAX_CONTENT_LEN
    + 400; // some padding for asset info

impl MaxSize for PostAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_POST_LEN)
    }
}
pub fn deserialize_post_account(data: &[u8]) -> Result<PostAccount> {
    if let AccountContainer::PostAccount(account) = try_from_slice_unchecked(data)? {
        return Ok(account);
    }
    panic!("Unkown data");
}
