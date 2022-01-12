use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, hash::hashv, pubkey::Pubkey};

use crate::shared::account::MaxSize;

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Description {
    String(String),
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct ChannelAccount {
    pub owner: Pubkey,
    pub name: String,
    pub description: Description,
}

impl ChannelAccount {
    pub fn new(owner: Pubkey, name: String, description: Description) -> ChannelAccount {
        ChannelAccount {
            owner,
            name,
            description,
        }
    }
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Message {
    String(String),
    // image
    // videos
    // files etc
}

impl Message {
    pub fn hash(&self) -> [u8; 32] {
        match &self {
            Message::String(string) => return hashv(&[string.as_bytes()]).to_bytes(),
        }
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub name: String,
}

impl MaxSize for UserAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

/*
pub type MessageAccountSplitted = (MessageAccount, Vec<String>);
pub enum MessageAccountSubmittable
{
    Split(MessageAccountSplitted),
    Single(MessageAccount)
}
 */

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct MessageAccount {
    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub message: Message,

    #[borsh_skip]
    pub size: u64,
}

impl MessageAccount {
    pub fn new(user: Pubkey, channel: Pubkey, timestamp: u64, message: Message) -> Self {
        match &message {
            Message::String(string) => {
                let message_size = string.as_bytes().len() as u64 + 4; // +4 because Borsh encodes length
                Self {
                    timestamp,
                    channel,
                    user,
                    message,
                    size: message_size,
                }
            }
        }
    }
}

impl MaxSize for MessageAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostAccount {
    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub spread_factor: Option<u64>,
    pub token: Pubkey,
    pub content: Pubkey,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostContentAccount {
    pub message: Message,
}

// Used to serialization and deserialization to keep track of account types
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]

pub enum AccountContainer {
    UserAccount(UserAccount),
    ChannelAccount(ChannelAccount),
    MessageAccount(MessageAccount),
    PostAccount(PostAccount),
    PostContentAccount(PostContentAccount),
}

impl MaxSize for AccountContainer {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

// Ugly helper methods below to reduce some bloat

pub fn deserialize_user_account(data: &[u8]) -> UserAccount {
    if let AccountContainer::UserAccount(account) = try_from_slice_unchecked(data).unwrap() {
        return account;
    }
    panic!();
}

pub fn deserialize_channel_account(data: &[u8]) -> ChannelAccount {
    if let AccountContainer::ChannelAccount(account) = try_from_slice_unchecked(data).unwrap() {
        return account;
    }
    panic!();
}

pub fn deserialize_message_account(data: &[u8]) -> MessageAccount {
    if let AccountContainer::MessageAccount(account) = try_from_slice_unchecked(data).unwrap() {
        return account;
    }
    panic!();
}

pub fn deserialize_post_account(data: &[u8]) -> PostAccount {
    if let AccountContainer::PostAccount(account) = try_from_slice_unchecked(data).unwrap() {
        return account;
    }
    panic!();
}
