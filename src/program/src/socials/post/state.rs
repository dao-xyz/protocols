use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use crate::{
    instruction::S2GAccountType,
    socials::{state::AccountType, MaxSize},
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ContentSource {
    External { url: String }, // like ipfs
}

pub const MAX_CONTENT_LEN: usize = 32 // hash pubkey
    + 200; // IPFS link (and some padding)

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Asset {
    NonAsset, // Not for sale, just a regular "post" (no one would want to buy this)
              // Add more markets here, like auction, then this would describe the owner token etc
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostType {
    SimplePost(InformationPost),
    ActionPost(ActionPost),
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostAccount {
    pub account_type: S2GAccountType,
    pub social_account_type: AccountType,
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub utility_mint_address: Pubkey, // either utility mint or goverence mint
    pub hash: [u8; 32],
    pub post_type: PostType,
    pub source: ContentSource,
    pub asset: Asset,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct InformationPost {
    pub created_at: u64,
    pub updated_at: u64,
    pub upvotes: u64,
    pub downvotes: u64,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VoteRule {
    Majority {
        upvote_numerator: u64, // upvote rate has to be atleast this fraction
        upvote_denominator: u64,
        downvote_numerator: u64, // downvote rate can not be more than this
        downvote_denominator: u64,
    },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ActionRule {
    pub channel: Pubkey,
    pub name: String,
    pub action: ActionType,
    pub rule: VoteRule,
    pub info: Option<String>,
}
impl ActionRule {
    pub fn is_approved(&self, action: &ActionPost, total_supply: u64) -> Option<bool> {
        match self.rule {
            VoteRule::Majority {
                downvote_denominator,
                downvote_numerator,
                upvote_denominator,
                upvote_numerator,
            } => {
                // upvotes/total_supply > upvote_numerator / upvote_denominator
                // upvotes * upvote_denominator > upvote_numerator * total_supply
                if !action
                    .upvotes
                    .checked_mul(upvote_denominator)?
                    .ge(&upvote_numerator.checked_mul(total_supply)?)
                {
                    return Some(false); // to few upvotes
                }

                // downvotes / total_supply > downvote_numerator / downvote_denominator
                // downvotes * downvote_denominator > downvote_numerator * total_supply
                if action
                    .downvotes
                    .checked_mul(downvote_denominator)?
                    .ge(&downvote_numerator.checked_mul(total_supply)?)
                {
                    return Some(false); // to many downvotes
                }
                return Some(true);
            }
        }
    }
}

pub fn deserialize_action_rule_account(data: &[u8]) -> Result<ActionRule> {
    let post_account: ActionRule = try_from_slice_unchecked(data)?;
    return Ok(post_account);
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VotingRuleUpdate {
    DeleteEventType(Pubkey),
    CreateEventType(ActionRule),
    UpdateEventType {
        rule: VoteRule,
        info: Option<String>,
    },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VotingRuleUpdateType {
    DeleteEventType,
    CreateEventType,
    UpdateEventType,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]

pub enum Action {
    Event {
        event_type: Pubkey,
        data: Vec<u8>,
    },
    ManageRule(VotingRuleUpdate),
    TransferTreasury {
        from: Pubkey, // treasury source
        to: Pubkey,   // reciever
        amount: u64,
    },
    DeletePost(Pubkey),
    SelfDestruct,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ActionType {
    Event(Pubkey), // event pubkey
    ManageRuleType(VotingRuleUpdateType),
    TransferTreasury,
    DeletePost,
    SelfDestruct,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ActionStatus {
    Pending,
    Rejected,
    Approved,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct ActionPost {
    pub created_at: u64,
    pub updated_at: u64,
    pub upvotes: u64,
    pub downvotes: u64,
    pub expires_at: u64,
    pub status: ActionStatus,
    pub action: Action,
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
pub fn deserialize_post_account(data: &[u8]) -> Result<PostAccount> {
    let post_account: PostAccount = try_from_slice_unchecked(data)?;
    return Ok(post_account);
}
