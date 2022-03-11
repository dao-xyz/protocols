use crate::{accounts::AccountType, error::PostError};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct RuleVoteWeight {
    pub account_type: AccountType,
    pub rule: Pubkey,
    pub mint: Pubkey,
    pub weight: u64,
}

pub fn get_rule_vote_weight_data(
    program_id: &Pubkey,
    rule_vote_weight_info: &AccountInfo,
    rule: &Pubkey,
) -> Result<RuleVoteWeight, ProgramError> {
    let data = get_account_data::<RuleVoteWeight>(program_id, rule_vote_weight_info)?;
    if &data.rule == rule {
        return Err(PostError::InvalidVoteRule.into());
    }
    Ok(data)
}

impl MaxSize for RuleVoteWeight {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for RuleVoteWeight {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::RuleVoteWeight
    }
}

pub fn find_rule_vote_weight_program_address(
    program_id: &Pubkey,
    rule_id: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[rule_id.as_ref(), mint.as_ref()], program_id)
}
pub fn create_rule_vote_weight_program_address_seeds<'a>(
    rule_id: &'a Pubkey,
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    return [rule_id.as_ref(), mint.as_ref(), bump_seed];
}
