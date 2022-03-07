use super::OptionVoteResult;
use crate::{
    accounts::AccountType,
    error::PostError,
    state::rules::{rule::Rule, rule_weight::RuleWeight},
};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum ProposalOptionType {
    Instruction {
        /// The number of the transactions already executed
        transactions_executed_count: u16,

        /// The number of transactions included in the option
        transactions_count: u16,

        /// The index of the the next transaction to be added
        transactions_next_index: u16,

        /// Option label
        label: String,
    },
    Deny,
}

/// Proposal Option
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct ProposalOption {
    pub account_type: AccountType,

    /// Option adhere to proposal
    pub proposal: Pubkey,

    /// Option index
    pub index: u16,

    /// Option type
    pub option_type: ProposalOptionType,

    /// Weights
    pub vote_weights: Vec<RuleWeight>,

    /// Vote result for the option
    pub vote_result: OptionVoteResult,
}
impl ProposalOption {
    pub fn add_weight(
        &mut self,
        amount: u64,
        vote_mint: &Pubkey,
        rule: &Pubkey,
        rule_data: &Rule,
    ) -> Result<(), ProgramError> {
        // Find mint weight
        for rule_mint_weight in &rule_data.vote_config.mint_weights {
            if &rule_mint_weight.mint == vote_mint {
                // Find rule vote weight to modify
                for vote_weight in &mut self.vote_weights {
                    if &vote_weight.rule == rule {
                        vote_weight.weight = vote_weight
                            .weight
                            .checked_add(amount.checked_mul(vote_weight.weight).unwrap())
                            .unwrap();
                        return Ok(());
                    }
                }
            }
        }
        Err(PostError::InvalidVoteMint.into())
    }
}

impl MaxSize for ProposalOption {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}
impl IsInitialized for ProposalOption {
    fn is_initialized(&self) -> bool {
        return self.account_type == AccountType::ProposalOption;
    }
}

pub fn get_proposal_option_data(
    program_id: &Pubkey,
    proposal_option_info: &AccountInfo,
    proposal: &Pubkey,
) -> Result<ProposalOption, ProgramError> {
    let data = get_account_data::<ProposalOption>(program_id, proposal_option_info)?;
    if &data.proposal != proposal {
        return Err(PostError::InvalidProposalForOption.into());
    }
    Ok(data)
}

pub fn find_proposal_option_program_address(
    program_id: &Pubkey,
    proposal: &Pubkey,
    option_index: &[u8; 2],
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[proposal.as_ref(), option_index], program_id)
}
pub fn create_proposal_option_program_address_seeds<'a>(
    proposal: &'a Pubkey,
    option_index: &'a [u8; 2],
) -> [&'a [u8]; 2] {
    return [proposal.as_ref(), option_index];
}
