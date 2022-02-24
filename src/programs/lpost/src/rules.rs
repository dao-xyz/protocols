use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{content::ContentSource, pack::MaxSize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

use std::io::Result;

use crate::accounts::AccountType;

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
pub enum RuleUpdateType {
    Create,
    Delete,
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

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ActionRule {
    pub account_type: AccountType,
    pub channel: Pubkey,
    pub deleted: bool,
    pub action: ActionType,
    pub criteria: AcceptenceCriteria,
    pub name: Option<String>,
    pub info: Option<ContentSource>,
}
impl MaxSize for ActionRule {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}
impl ActionRule {
    pub fn is_approved(&self, upvotes: u64, downvotes: u64, total_supply: u64) -> Option<bool> {
        match self.criteria {
            AcceptenceCriteria::Majority {
                downvote_denominator,
                downvote_numerator,
                upvote_denominator,
                upvote_numerator,
            } => {
                // upvotes/total_supply >= upvote_numerator / upvote_denominator
                // upvotes * upvote_denominator > upvote_numerator * total_supply
                if upvotes
                    .checked_mul(upvote_denominator)?
                    .le(&upvote_numerator.checked_mul(total_supply)?)
                {
                    return Some(false); // to few upvotes
                }

                // downvotes / total_supply >= downvote_numerator / downvote_denominator
                // downvotes * downvote_denominator > downvote_numerator * total_supply
                if !downvotes
                    .checked_mul(downvote_denominator)?
                    .le(&downvote_numerator.checked_mul(total_supply)?)
                {
                    return Some(false); // to many downvotes
                }
                Some(true)
            }
        }
    }
}

pub fn deserialize_action_rule_account(data: &[u8]) -> Result<ActionRule> {
    let post_account: ActionRule = try_from_slice_unchecked(data)?;
    Ok(post_account)
}
