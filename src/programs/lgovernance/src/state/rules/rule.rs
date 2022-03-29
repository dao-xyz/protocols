use std::slice::Iter;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use ltag::state::get_tag_record_data_with_authority_and_signed_owner;
use shared::{
    account::{get_account_data, MaxSize},
    content::ContentSource,
};
use solana_program::{
    account_info::{next_account_info, next_account_infos, AccountInfo},
    msg,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        proposal::{proposal_transaction::InstructionData, ProposalV2},
        token_owner_record::get_token_owner_record_data_for_owner,
    },
    tokens::spl_utils::{get_spl_token_mint_supply, get_token_balance},
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
impl RuleTimeConfig {
    pub fn get_strictest(&self, other: &Self) -> Self {
        Self {
            max_voting_time: self.max_voting_time.max(other.max_voting_time),
            min_transaction_hold_up_time: self
                .min_transaction_hold_up_time
                .max(other.min_transaction_hold_up_time),
            proposal_cool_off_time: self
                .proposal_cool_off_time
                .max(other.proposal_cool_off_time),
        }
    }
}

impl Default for RuleTimeConfig {
    fn default() -> Self {
        Self {
            min_transaction_hold_up_time: 0,
            max_voting_time: 604800,
            proposal_cool_off_time: 0,
        }
    }
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
pub struct RuleProposalConfig {
    pub create_proposal_criteria: CreateProposalCriteria,
}

impl RuleProposalConfig {
    pub fn assert_can_create_proposal(
        &self,
        program_id: &Pubkey,
        proposal: &ProposalV2,
        accounts: &mut Iter<AccountInfo>,
    ) -> Result<(), ProgramError> {
        match &self.create_proposal_criteria {
            CreateProposalCriteria::AuthorityTag { authority, tag } => {
                let tag_record_info = next_account_info(accounts)?;
                let tag_record_owner = next_account_info(accounts)?;
                let tag_record_data = get_tag_record_data_with_authority_and_signed_owner(
                    &ltag::id(),
                    tag_record_info,
                    authority,
                    tag_record_owner,
                )?;
                if &tag_record_data.tag != tag {
                    return Err(GovernanceError::InvalidTagRecord.into());
                }
                Ok(())
            }
            CreateProposalCriteria::TokenOwner { amount, mint } => {
                let token_owner_record = next_account_info(accounts)?;
                let governing_token_owner_info = next_account_info(accounts)?;

                let token_owner_record_data = get_token_owner_record_data_for_owner(
                    program_id,
                    token_owner_record,
                    governing_token_owner_info,
                )?;

                if &token_owner_record_data.governing_token_mint != mint {
                    return Err(GovernanceError::InvalidGoverningMintForProposal.into());
                }

                if amount < &token_owner_record_data.governing_token_deposit_amount {
                    return Err(GovernanceError::InvalidTokenBalance.into());
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct RuleConfig {
    pub vote_config: RuleVoteConfig,
    pub time_config: RuleTimeConfig,
    pub proposal_config: RuleProposalConfig,
}

impl RuleConfig {
    pub fn get_single_mint_config(
        governance_mint: &Pubkey,
        rule_condition: &Option<RuleCondition>,
        name: &Option<String>,
        info: &Option<ContentSource>,
    ) -> Self {
        Self {
            proposal_config: RuleProposalConfig {
                create_proposal_criteria: CreateProposalCriteria::TokenOwner {
                    amount: 1,
                    mint: *governance_mint,
                },
            },
            time_config: RuleTimeConfig::default(),
            vote_config: RuleVoteConfig {
                criteria: AcceptenceCriteria::default(),
                info: info.clone(),
                mint_weights: vec![MintWeight {
                    mint: *governance_mint,
                    weight: 100,
                }],
                name: name.clone(),
                rule_condition: rule_condition.clone(),
                vote_tipping: VoteTipping::Strict,
            },
        }
    }
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum CreateProposalCriteria {
    AuthorityTag { tag: Pubkey, authority: Pubkey },
    TokenOwner { mint: Pubkey, amount: u64 },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct Rule {
    pub account_type: AccountType,

    // config
    // id is basically a seed (has to be unique)
    pub id: Pubkey,
    pub governance: Pubkey,
    pub deleted: bool,

    // config
    pub config: RuleConfig,

    // stats
    pub proposal_count: u64,
    pub voting_proposal_count: u64,
}
impl MaxSize for Rule {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl Rule {
    pub fn rule_applicable(&self, instruction_data: &InstructionData) -> Result<(), ProgramError> {
        if let Some(config) = &self.config.vote_config.rule_condition {
            match config {
                RuleCondition::None => return Ok(()),
                RuleCondition::ProgramId(program_id) => {
                    if program_id != &instruction_data.program_id {
                        return Err(GovernanceError::RuleNotApplicableForInstruction.into());
                    }
                    return Ok(());
                }
                RuleCondition::Granular {
                    program_id,
                    instruction_condition,
                } => {
                    if program_id != &instruction_data.program_id {
                        return Err(GovernanceError::RuleNotApplicableForInstruction.into());
                    }

                    // Check instruction data
                    for chunk in &instruction_condition.chunks {
                        for (i, chunk_data) in chunk.data.iter().enumerate() {
                            if instruction_data
                                .data
                                .get(chunk.offset as usize + i)
                                .unwrap()
                                != chunk_data
                            {
                                return Err(GovernanceError::RuleNotApplicableForInstruction.into());
                            }
                        }
                    }
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

impl IsInitialized for Rule {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Rule
    }
}

/// Deserializes Rule account and governance key
pub fn get_rule_data_for_governance(
    program_id: &Pubkey,
    rule_info: &AccountInfo,
    governance: &Pubkey,
) -> Result<Rule, ProgramError> {
    let data = get_account_data::<Rule>(program_id, rule_info)?;
    if &data.governance != governance {
        return Err(GovernanceError::InvalidGovernanceForRule.into());
    }
    Ok(data)
}

/// Deserializes Rule account
pub fn get_rule_data(program_id: &Pubkey, rule_info: &AccountInfo) -> Result<Rule, ProgramError> {
    let data = get_account_data::<Rule>(program_id, rule_info)?;
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
                return Err(GovernanceError::InvalidVoteMint.into());
            }
            let supply = get_spl_token_mint_supply(mint_info)?;
            sum = sum
                .checked_add(supply.checked_mul(mint_weight.weight).unwrap())
                .unwrap();
        }
        Ok(sum)
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

                true
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
pub fn get_rule_program_address(program_id: &Pubkey, rule_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[rule_id.as_ref()], program_id)
}
pub fn get_rule_program_address_seeds<'a>(
    rule_id: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 2] {
    return [rule_id.as_ref(), bump_seed];
}
