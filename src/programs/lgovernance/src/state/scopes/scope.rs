use std::slice::Iter;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use ltag::state::get_tag_record_data_with_factory_and_signed_owner;
use shared::{
    account::{get_account_data, MaxSize},
    content::ContentSource,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        proposal::{proposal_transaction::InstructionData, ProposalV2},
        vote_power_owner_record::get_vote_power_owner_record_data_for_owner,
    },
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
pub enum ScopeUpdateType {
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
    ManageScope(ScopeUpdateType),
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
pub enum ScopeMatch {
    // For what program id?
    All,
    ProgramId(Pubkey),
    Granular {
        program_id: Pubkey,
        instruction_condition: InstructionConditional,
    },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ScopeTimeConfig {
    pub min_transaction_hold_up_time: u32,

    /// Time limit in seconds for proposal to be open for voting
    pub max_voting_time: u32,

    /// The time period in seconds within which a Proposal can be still cancelled after being voted on
    /// Once cool off time expires Proposal can't be cancelled any longer and becomes a law
    /// Note: This field is not implemented in the current version
    pub proposal_cool_off_time: u32,
}
/*
impl ScopeTimeConfig {
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
 */
impl Default for ScopeTimeConfig {
    fn default() -> Self {
        Self {
            min_transaction_hold_up_time: 0,
            max_voting_time: 604800,
            proposal_cool_off_time: 0,
        }
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VotePowerUnit {
    Mint(Pubkey),
    Tag { record_factory: Pubkey },
}

impl VotePowerUnit {
    pub fn assert_compatible(&self, other: &Self) -> Result<(), ProgramError> {
        match self {
            Self::Mint(governing_token_mint) => {
                if let Self::Mint(other_governing_token_mint) = other {
                    if governing_token_mint == other_governing_token_mint {
                        return Ok(());
                    }
                }
            }
            Self::Tag { record_factory, .. } => {
                if let Self::Tag {
                    record_factory: other_record_factory,
                    ..
                } = other
                {
                    if record_factory == other_record_factory {
                        return Ok(());
                    }
                }
            }
        }
        Err(GovernanceError::InvalidVotePowerSource.into())
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct SourceWeight {
    pub source: VotePowerUnit,
    pub weight: u64,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ScopeVoteConfig {
    pub scope_condition: Option<ScopeMatch>,
    pub criteria: AcceptenceCriteria,
    pub source_weights: Vec<SourceWeight>,
    /// Conditions under which a vote will complete early
    pub vote_tipping: VoteTipping,

    pub name: Option<String>,
    pub info: Option<ContentSource>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ScopeProposalConfig {
    pub create_proposal_criteria: CreateProposalCriteria,
}

impl ScopeProposalConfig {
    pub fn assert_can_create_proposal(
        &self,
        program_id: &Pubkey,
        _proposal: &ProposalV2,
        accounts: &mut Iter<AccountInfo>,
    ) -> Result<(), ProgramError> {
        match &self.create_proposal_criteria {
            CreateProposalCriteria::Tag { record_factory } => {
                let tag_record_info = next_account_info(accounts)?;
                let tag_record_owner = next_account_info(accounts)?;
                let _tag_record_data = get_tag_record_data_with_factory_and_signed_owner(
                    &ltag::id(),
                    tag_record_info,
                    record_factory,
                    tag_record_owner,
                )?;
                Ok(())
            }
            CreateProposalCriteria::Token { amount, mint } => {
                let token_owner_record = next_account_info(accounts)?;
                let governing_owner_info = next_account_info(accounts)?;
                let token_owner_record_data = get_vote_power_owner_record_data_for_owner(
                    program_id,
                    token_owner_record,
                    governing_owner_info,
                )?;

                if let VotePowerUnit::Mint(governing_token_mint) = &token_owner_record_data.source {
                    if governing_token_mint != mint {
                        return Err(GovernanceError::InvalidGoverningMintForProposal.into());
                    }

                    if amount < &token_owner_record_data.amount {
                        return Err(GovernanceError::InvalidTokenBalance.into());
                    }
                    Ok(())
                } else {
                    Err(GovernanceError::InvalidVotePowerSource.into())
                }
            }
        }
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct ScopeConfig {
    pub vote_config: ScopeVoteConfig,
    pub time_config: ScopeTimeConfig,
    pub proposal_config: ScopeProposalConfig,
}

impl ScopeConfig {
    pub fn get_single_mint_config(
        governance_mint: &Pubkey,
        scope_condition: &Option<ScopeMatch>,
        name: &Option<String>,
        info: &Option<ContentSource>,
    ) -> Self {
        Self {
            proposal_config: ScopeProposalConfig {
                create_proposal_criteria: CreateProposalCriteria::Token {
                    amount: 1,
                    mint: *governance_mint,
                },
            },
            time_config: ScopeTimeConfig::default(),
            vote_config: ScopeVoteConfig {
                criteria: AcceptenceCriteria::default(),
                info: info.clone(),
                source_weights: vec![SourceWeight {
                    source: VotePowerUnit::Mint(*governance_mint),
                    weight: 100,
                }],
                name: name.clone(),
                scope_condition: scope_condition.clone(),
                vote_tipping: VoteTipping::Strict,
            },
        }
    }

    pub fn get_single_tag_config(
        record_factory: &Pubkey,
        scope_condition: &Option<ScopeMatch>,
        name: &Option<String>,
        info: &Option<ContentSource>,
    ) -> Self {
        Self {
            proposal_config: ScopeProposalConfig {
                create_proposal_criteria: CreateProposalCriteria::Tag {
                    record_factory: *record_factory,
                },
            },
            time_config: ScopeTimeConfig::default(),
            vote_config: ScopeVoteConfig {
                criteria: AcceptenceCriteria::default(),
                info: info.clone(),
                source_weights: vec![SourceWeight {
                    source: VotePowerUnit::Tag {
                        record_factory: *record_factory,
                    },
                    weight: 100,
                }],
                name: name.clone(),
                scope_condition: scope_condition.clone(),
                vote_tipping: VoteTipping::Strict,
            },
        }
    }
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum CreateProposalCriteria {
    Tag { record_factory: Pubkey },
    Token { mint: Pubkey, amount: u64 },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct Scope {
    pub account_type: AccountType,

    // config
    // id is basically a seed (has to be unique)
    pub id: Pubkey,
    pub governance: Pubkey,
    pub deleted: bool,

    // config
    pub config: ScopeConfig,

    // stats
    pub proposal_count: u64,
    pub voting_proposal_count: u64,
}
impl MaxSize for Scope {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl Scope {
    pub fn scope_applicable(&self, instruction_data: &InstructionData) -> Result<(), ProgramError> {
        if let Some(config) = &self.config.vote_config.scope_condition {
            match config {
                ScopeMatch::All => return Ok(()),
                ScopeMatch::ProgramId(program_id) => {
                    if program_id != &instruction_data.program_id {
                        return Err(GovernanceError::ScopeNotApplicableForInstruction.into());
                    }
                    return Ok(());
                }
                ScopeMatch::Granular {
                    program_id,
                    instruction_condition,
                } => {
                    if program_id != &instruction_data.program_id {
                        return Err(GovernanceError::ScopeNotApplicableForInstruction.into());
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
                                return Err(
                                    GovernanceError::ScopeNotApplicableForInstruction.into()
                                );
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

impl IsInitialized for Scope {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Scope
    }
}

/// Deserializes Scope account and governance key
pub fn get_scope_data_for_governance(
    program_id: &Pubkey,
    scope_info: &AccountInfo,
    governance: &Pubkey,
) -> Result<Scope, ProgramError> {
    let data = get_account_data::<Scope>(program_id, scope_info)?;
    if &data.governance != governance {
        return Err(GovernanceError::InvalidGovernanceForscope.into());
    }
    Ok(data)
}

/// Deserializes Scope account
pub fn get_scope_data(
    program_id: &Pubkey,
    scope_info: &AccountInfo,
) -> Result<Scope, ProgramError> {
    let data = get_account_data::<Scope>(program_id, scope_info)?;
    Ok(data)
}

impl ScopeVoteConfig {
    /*
     pub fn max_vote_weight(
        &self,
        accounts_iter: &mut Iter<AccountInfo>,
    ) -> Result<u64, ProgramError> {
        let mut sum = 0_u64;
        for source_weight in &self.source_weights {
            match &source_weight.source {
                VoteSource::Mint(mint) => {
                    let mint_info = next_account_info(accounts_iter)?;
                    if mint_info.key != mint {
                        return Err(GovernanceError::InvalidVoteMint.into());
                    }
                    let supply = get_spl_token_mint_supply(mint_info)?;
                    sum = sum
                        .checked_add(supply.checked_mul(source_weight.weight).unwrap())
                        .unwrap();
                }
                VoteSource::Tag { record_factory } => {
                    let record_factory_info = next_account_info(accounts_iter)?;
                    if record_factory_info.key != record_factory {
                        return Err(GovernanceError::InvalidTagRecordFactory.into());
                    }

                    let tag_record_factory = get_account_data::<TagRecordFactoryAccount>(
                        &ltag::id(),
                        record_factory_info,
                    )?;

                    let supply = tag_record_factory.outstanding_records;
                    sum = sum
                        .checked_add(supply.checked_mul(source_weight.weight).unwrap())
                        .unwrap();
                }
            }
        }
        Ok(sum)
    }
    */
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

/* impl Scope {
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
pub fn get_scope_program_address(program_id: &Pubkey, scope_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[scope_id.as_ref()], program_id)
}
pub fn get_scope_program_address_seeds<'a>(
    scope_id: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 2] {
    return [scope_id.as_ref(), bump_seed];
}
