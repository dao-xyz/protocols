

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};


use shared::account::get_account_data;
use shared::{account::MaxSize, content::ContentSource};
use solana_program::account_info::{AccountInfo};
use solana_program::clock::UnixTimestamp;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::IsInitialized;
use solana_program::pubkey::Pubkey;

use crate::accounts::AccountType;
use crate::error::PostError;

pub const MAX_CONTENT_LEN: usize = 32 // hash pubkey
    + 200; // IPFS link (and some padding)

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostContent {
    Info { content: ContentSource },
    External { program_id: Pubkey, id: Pubkey },
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct InformationPost {}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum VoteConfig {
    Simple { upvote: u64, downvote: u64 },
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostAccount {
    pub account_type: AccountType,
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub hash: [u8; 32],
    pub create_at_timestamp: UnixTimestamp,
    pub vote_config: VoteConfig,
    pub content: PostContent,
    pub deleted_at_timestamp: Option<UnixTimestamp>,
}

impl IsInitialized for PostAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Post
    }
}

pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_POST_LEN: usize = 32 // creator pubkey
    + 32 // channel pubkey
    + 8 // timestamp
    + MAX_CONTENT_LEN
    + 400; // some padding for asset info

impl MaxSize for PostAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_POST_LEN)
    }
}

pub fn get_post_data(
    program_id: &Pubkey,
    post_account_info: &AccountInfo,
    channel: &Pubkey,
) -> Result<PostAccount, ProgramError> {
    let data = get_account_data::<PostAccount>(program_id, post_account_info)?;
    if &data.channel != channel {
        return Err(PostError::InvalidPostForChannel.into());
    }
    Ok(data)
}
