use std::slice::Iter;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{
    account::{get_account_data, MaxSize},
    content::ContentSource,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::{
    accounts::AccountType, error::PostError, tokens::spl_utils::get_spl_token_mint_supply,
};

use super::super::enums::VoteTipping;

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum AcceptenceCriteria {
    Threshold { numerator: u64, denominator: u64 },
}

impl Default for AcceptenceCriteria {
    fn default() -> Self {
        // Atleast 50% upvotes
        Self::Threshold {
            denominator: 100,
            numerator: 50,
        }
    }
}
/*
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
}*/

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct InstructionChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct InstructionConditional {
    pub chunks: Vec<InstructionChunk>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum RuleCondition {
    // For what program id?
    None,
    ProgramId(Pubkey),
    Granular {
        program_id: Pubkey,
        instruction_condition: InstructionConditional,
    },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct RuleTimeConfig {
    pub min_transaction_hold_up_time: u32,

    /// Time limit in seconds for proposal to be open for voting
    pub max_voting_time: u32,

    /// The time period in seconds within which a Proposal can be still cancelled after being voted on
    /// Once cool off time expires Proposal can't be cancelled any longer and becomes a law
    /// Note: This field is not implemented in the current version
    pub proposal_cool_off_time: u32,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct MintWeight {
    pub mint: Pubkey,
    pub weight: u64,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct RuleVoteConfig {
    pub rule_condition: Option<RuleCondition>,
    pub criteria: AcceptenceCriteria,
    pub mint_weights: Vec<MintWeight>,
    /// Conditions under which a vote will complete early
    pub vote_tipping: VoteTipping,

    pub name: Option<String>,
    pub info: Option<ContentSource>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct Rule {
    pub account_type: AccountType,
    // config
    // id is basically a seed (has to be unique)
    pub id: Pubkey,
    pub channel: Pubkey,
    pub deleted: bool,

    // config
    pub vote_config: RuleVoteConfig,
    pub time_config: RuleTimeConfig,

    // stats
    pub voting_proposal_count: u64,
}
impl MaxSize for Rule {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for Rule {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Rule
    }
}

/// Deserializes Rule account and checks channel and owner program
pub fn get_rule_data(
    program_id: &Pubkey,
    rule_info: &AccountInfo,
    channel: &Pubkey,
) -> Result<Rule, ProgramError> {
    let data = get_account_data::<Rule>(program_id, rule_info)?;
    if &data.channel == channel {
        return Err(PostError::InvalidChannel.into());
    }
    Ok(data)
}

impl RuleVoteConfig {
    pub fn max_vote_weight(
        &self,
        accounts_iter: &mut Iter<AccountInfo>,
    ) -> Result<u64, ProgramError> {
        let mut sum = 0_u64;
        for mint_weight in &self.mint_weights {
            let mint_info = next_account_info(accounts_iter)?;
            if mint_info.key != &mint_weight.mint {
                return Err(PostError::InvalidVoteMint.into());
            }
            let supply = get_spl_token_mint_supply(mint_info)?;
            sum = sum
                .checked_add(supply.checked_mul(mint_weight.weight).unwrap())
                .unwrap();
        }
        return Ok(sum);
    }
    pub fn is_approved(
        &self,
        weight: u64,
        deny_vote_weight: Option<u64>,
        max_vote_weight: u64,
    ) -> bool {
        let deny_vote_weight = deny_vote_weight.unwrap_or(0);

        match &self.criteria {
            AcceptenceCriteria::Threshold {
                denominator,
                numerator,
            } => {
                if deny_vote_weight >= weight {
                    return false;
                }

                // upvotes/total_supply >= upvote_numerator / upvote_denominator
                // upvotes * upvote_denominator > upvote_numerator * total_supply
                if weight
                    .checked_mul(*denominator)
                    .unwrap()
                    .le(&numerator.checked_mul(max_vote_weight).unwrap())
                {
                    return false; // to few upvotes
                }

                return true;
            }
        }
    }
}

/* impl Rule {
    pub fn is_approved(&self, upvotes: u64, downvotes: u64, total_supply: u64) -> Option<bool> {
        match self.vote_config.criteria {
            AcceptenceCriteria::Threshold(threshold) => {
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
} */
pub fn find_create_rule_associated_program_address(
    program_id: &Pubkey,
    rule_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[rule_id.as_ref()], program_id)
}
pub fn create_rule_associated_program_address_seeds<'a>(
    rule_id: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 2] {
    return [rule_id.as_ref(), bump_seed];
}
