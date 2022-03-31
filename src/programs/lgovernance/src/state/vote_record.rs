//! Proposal Vote Record Account

use borsh::maybestd::io::Write;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::account_info::AccountInfo;

use solana_program::program_error::ProgramError;
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

use crate::accounts::AccountType;
use crate::error::GovernanceError;

use crate::PROGRAM_AUTHORITY_SEED;

/// Voter choice for a proposal option
/// In the current version only 1) Single choice and 2) Multiple choices proposals are supported
/// In the future versions we can add support for 1) Quadratic voting, 2) Ranked choice voting and 3) Weighted voting
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct VoteChoice {
    /// The rank given to the choice by voter
    /// Note: The filed is not used in the current version
    pub rank: u8,

    /// The voter's weight percentage given by the voter to the choice
    pub weight_percentage: u8,
}

impl VoteChoice {
    /// Returns the choice weight given the voter's weight
    pub fn get_choice_weight(&self, voter_weight: u64) -> Result<u64, ProgramError> {
        Ok(match self.weight_percentage {
            100 => voter_weight,
            0 => 0,
            _ => return Err(GovernanceError::InvalidVoteChoiceWeightPercentage.into()),
        })
    }
}

/// Vote option indices
pub type Vote = Vec<u16>;

/// Proposal VoteRecord
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct VoteRecordV2 {
    /// Governance account type
    pub account_type: AccountType,

    /// Proposal account
    pub proposal: Pubkey,

    /// Last vote made
    pub previous_vote: Option<Pubkey>,

    /// Next vote
    pub next_vote: Option<Pubkey>,

    /// The user who casted this vote
    /// This is the Governing Token Owner who deposited governing tokens into the Realm
    pub governing_token_owner: Pubkey,

    /// The voting has been made through this scope
    pub scope: Pubkey,

    /// Indicates whether the vote was relinquished by voter
    pub is_relinquished: bool,

    /// Voter's vote
    pub vote: Vote,
}

impl MaxSize for VoteRecordV2 {}

impl IsInitialized for VoteRecordV2 {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::VoteRecordV2
    }
}
impl VoteRecordV2 {
    /// Checks the vote can be relinquished
    pub fn assert_can_relinquish_vote(&self) -> Result<(), ProgramError> {
        if self.is_relinquished {
            return Err(GovernanceError::VoteAlreadyRelinquished.into());
        }

        Ok(())
    }

    /// Serializes account into the target buffer
    pub fn serialize<W: Write>(self, writer: &mut W) -> Result<(), ProgramError> {
        BorshSerialize::serialize(&self, writer)?;
        Ok(())
    }

    pub fn assert_vote_equals(&self, other: &Vote) -> Result<(), ProgramError> {
        if self.vote.len() != other.len() {
            return Err(GovernanceError::InvalidVote.into());
        }
        for (a, b) in self.vote.iter().zip(other.iter().as_ref()) {
            if a != b {
                return Err(GovernanceError::InvalidVote.into());
            }
        }
        Ok(())
    }
}

/// Deserializes VoteRecord account and checks owner program
pub fn get_vote_record_data(
    program_id: &Pubkey,
    vote_record_info: &AccountInfo,
) -> Result<VoteRecordV2, ProgramError> {
    get_account_data::<VoteRecordV2>(program_id, vote_record_info)
}

/// Deserializes VoteRecord and checks it belongs to the provided Proposal and Governing Token Owner
pub fn get_vote_record_data_for_proposal_and_token_owner(
    program_id: &Pubkey,
    vote_record_info: &AccountInfo,
    proposal: &Pubkey,
    governing_token_owner: &AccountInfo,
) -> Result<VoteRecordV2, ProgramError> {
    let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;

    if vote_record_data.proposal != *proposal {
        return Err(GovernanceError::InvalidProposalForVoterRecord.into());
    }

    if &vote_record_data.governing_token_owner != governing_token_owner.key {
        return Err(GovernanceError::InvalidGoverningTokenOwnerForVoteRecord.into());
    }

    if !governing_token_owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(vote_record_data)
}

/// Deserializes VoteRecord and checks it belongs to the provided Proposal and unsigned Governing Token Owner
pub fn get_vote_record_data_for_proposal_and_unsigned_token_owner(
    program_id: &Pubkey,
    vote_record_info: &AccountInfo,
    proposal: &Pubkey,
    governing_token_owner: &Pubkey,
) -> Result<VoteRecordV2, ProgramError> {
    let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;

    if vote_record_data.proposal != *proposal {
        return Err(GovernanceError::InvalidProposalForVoterRecord.into());
    }

    if &vote_record_data.governing_token_owner != governing_token_owner {
        return Err(GovernanceError::InvalidGoverningTokenOwnerForVoteRecord.into());
    }

    Ok(vote_record_data)
}

/// Returns VoteRecord PDA seeds
pub fn get_vote_record_address_seeds<'a>(
    proposal: &'a Pubkey,
    token_owner_record: &'a Pubkey,
    scope: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 5] {
    [
        PROGRAM_AUTHORITY_SEED,
        proposal.as_ref(),
        token_owner_record.as_ref(),
        scope.as_ref(),
        bump_seed,
    ]
}

/// Returns VoteRecord PDA address
pub fn get_vote_record_address(
    program_id: &Pubkey,
    proposal: &Pubkey,
    token_owner_record: &Pubkey,
    scope: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PROGRAM_AUTHORITY_SEED,
            proposal.as_ref(),
            token_owner_record.as_ref(),
            scope.as_ref(),
        ],
        program_id,
    )
}
