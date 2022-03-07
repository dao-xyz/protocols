use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::{
    accounts::AccountType, error::PostError, state::rules::rule_vote_weight::RuleVoteWeight,
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ProposalVoteWeight {
    pub account_type: AccountType,
    pub proposal: Pubkey,
    pub rule: Pubkey,
    pub option_index: u16,
    pub weight: u64,
}

impl ProposalVoteWeight {
    pub fn add_weight(
        &mut self,
        amount: u64,
        vote_mint: &Pubkey,
        rule_vote_weight: &RuleVoteWeight,
    ) -> Result<(), ProgramError> {
        if &rule_vote_weight.mint == vote_mint && rule_vote_weight.rule == self.rule {
            self.weight = amount.checked_add(rule_vote_weight.weight).unwrap();
            return Ok(());
        }

        Err(PostError::InvalidVoteMint.into())
    }
}

impl MaxSize for ProposalVoteWeight {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for ProposalVoteWeight {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::ProposalVoteWeight
    }
}

pub fn find_proposal_vote_weight_program_address(
    program_id: &Pubkey,
    proposal: &Pubkey,
    rule_id: &Pubkey,
    mint: &Pubkey,
    option_index: u8,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            proposal.as_ref(),
            rule_id.as_ref(),
            mint.as_ref(),
            &[option_index],
        ],
        program_id,
    )
}
pub fn create_proposal_vote_weight_program_address_seeds<'a>(
    proposal: &'a Pubkey,
    rule_id: &'a Pubkey,
    option_index: &'a [u8; 2],
    bump_seed: &'a [u8; 1],
) -> [&'a [u8]; 4] {
    return [proposal.as_ref(), rule_id.as_ref(), option_index, bump_seed];
}

/// Deserializes Proposal vote weight account and checks channel and owner program
pub fn get_proposal_vote_weight_data(
    program_id: &Pubkey,
    proposal_vote_weight_info: &AccountInfo,
    proposal: &Pubkey,
    rule: &Pubkey,
    option_index: &u16,
) -> Result<ProposalVoteWeight, ProgramError> {
    let data = get_account_data::<ProposalVoteWeight>(program_id, proposal_vote_weight_info)?;

    if &data.proposal == proposal {
        return Err(PostError::InvalidProposalForVoteWeight.into());
    }

    if &data.rule == rule {
        return Err(PostError::InvalidVoteRule.into());
    }

    if &data.option_index == option_index {
        return Err(PostError::InvalidOptionForVote.into());
    }

    Ok(data)
}
