use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use crate::{
    instruction::S2GAccountType,
    socials::{state::AccountType, MaxSize},
    tokens::spl_utils::find_authority_program_address,
};

use super::{find_create_rule_associated_program_address, find_treasury_token_account_address};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ContentSource {
    External { url: String },
    String(String),
}

impl From<&str> for ContentSource {
    fn from(string: &str) -> Self {
        return ContentSource::String(string.to_string());
    }
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
    InformalPost(InformationPost),
    ActionPost(ActionPost),
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostAccount {
    pub account_type: S2GAccountType,
    pub social_account_type: AccountType,
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub utility_mint_address: Pubkey, // either utility mint or goverence mint
    pub deleted: bool,
    pub hash: [u8; 32],
    pub post_type: PostType,
    pub source: ContentSource,
    pub asset: Asset,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct InformationPost {
    pub created_at: u64,
    pub upvotes: u64,
    pub downvotes: u64,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum AcceptenceCriteria {
    Majority {
        upvote_numerator: u64, // upvote rate has to be atleast this fraction
        upvote_denominator: u64,
        downvote_numerator: u64, // downvote rate can not be more than this
        downvote_denominator: u64,
    },
}

impl Default for AcceptenceCriteria {
    fn default() -> Self {
        // Atleast 50% upvotes,
        // and not more than 50% downvotes
        Self::Majority {
            downvote_denominator: 100,
            downvote_numerator: 50,
            upvote_denominator: 100,
            upvote_numerator: 50,
        }
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ActionRule {
    pub account_type: S2GAccountType,
    pub social_account_type: AccountType,
    pub channel: Pubkey,
    pub action: ActionType,
    pub criteria: AcceptenceCriteria,
    pub name: Option<String>,
    pub info: Option<ContentSource>,
    pub deleted: bool,
}
impl MaxSize for ActionRule {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}
impl ActionRule {
    pub fn is_approved(&self, action: &ActionPost, total_supply: u64) -> Option<bool> {
        match self.criteria {
            AcceptenceCriteria::Majority {
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
pub struct CreateRule {
    pub channel: Pubkey,
    pub action: ActionType,
    pub criteria: AcceptenceCriteria,
    pub name: Option<String>,
    pub info: Option<ContentSource>,
}

impl MaxSize for CreateRule {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VotingRuleUpdate {
    Delete(Pubkey),
    Create { rule: CreateRule, bump_seed: u8 },
}

impl VotingRuleUpdate {
    pub fn create(rule: CreateRule, channel: &Pubkey, program_id: &Pubkey) -> Self {
        let bump_seed =
            find_create_rule_associated_program_address(program_id, &rule.action, &channel).1;

        Self::Create { rule, bump_seed }
    }
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Action {
    CustomEvent { event_type: Pubkey, data: Vec<u8> },
    ManageRule(VotingRuleUpdate),
    Treasury(TreasuryAction),
    DeletePost(Pubkey),
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum TreasuryAction {
    Transfer {
        from: Pubkey,
        to: Pubkey,
        amount: u64,
        bump_seed: u8,
    },
    Create {
        mint: Pubkey,
    }, // mint
}

impl TreasuryAction {
    pub fn transfer(from: &Pubkey, to: &Pubkey, amount: u64, program_id: &Pubkey) -> Self {
        Self::Transfer {
            from: *from,
            to: *to,
            amount,
            bump_seed: find_authority_program_address(program_id, from).1,
        }
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum RuleUpdateType {
    Delete,
    Create,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum TreasuryActionType {
    Transfer {
        from: Option<Pubkey>,
        to: Option<Pubkey>,
    },
    Create,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ActionType {
    CustomEvent(Pubkey), // event pubkey
    ManageRule(RuleUpdateType),
    Treasury(TreasuryActionType),
    DeletePost,
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
