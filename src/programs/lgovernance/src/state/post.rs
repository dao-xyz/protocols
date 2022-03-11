use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{account::MaxSize, content::ContentSource};
use solana_program::clock::UnixTimestamp;
use solana_program::program_pack::IsInitialized;
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use super::proposal::ProposalV2;
use super::rules::rule::{find_create_rule_associated_program_address, AcceptenceCriteria};
use super::{rules::rule::InstructionConditional};

use crate::{accounts::AccountType};

pub const MAX_CONTENT_LEN: usize = 32 // hash pubkey
    + 200; // IPFS link (and some padding)

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostType {
    InformationPost(InformationPost),
    Proposal(ProposalV2),
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct InformationPost {
    pub created_at: UnixTimestamp,
    pub upvotes: u64,
    pub downvotes: u64,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostAccount {
    pub account_type: AccountType,
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub deleted: bool,
    pub hash: [u8; 32],
    pub post_type: PostType,
    pub source: ContentSource,
}

impl IsInitialized for PostAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Post
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct CreateRule {
    pub id: Pubkey,
    pub channel: Pubkey,
    pub vote_mint: Pubkey,
    /*     pub action: ActionType, */
    pub instruction_program_id: Pubkey,
    pub instruction_condition: InstructionConditional,
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
    pub fn create(rule: CreateRule, program_id: &Pubkey) -> Self {
        let bump_seed = find_create_rule_associated_program_address(program_id, &rule.id).1;
        Self::Create { rule, bump_seed }
    }
}
/*
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
}*/

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
    Ok(post_account)
}
