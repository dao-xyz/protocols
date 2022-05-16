use crate::{accounts::AccountType, error::PostError};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ScopeVoteWeight {
    pub account_type: AccountType,
    pub scope: Pubkey,
    pub mint: Pubkey,
    pub weight: u64,
}

pub fn get_scope_vote_weight_data(
    program_id: &Pubkey,
    scope_vote_weight_info: &AccountInfo,
    scope: &Pubkey,
) -> Result<ScopeVoteWeight, ProgramError> {
    let data = get_account_data::<ScopeVoteWeight>(program_id, scope_vote_weight_info)?;
    if &data.scope == scope {
        return Err(PostError::InvalidVotescope.into());
    }
    Ok(data)
}

impl MaxSize for ScopeVoteWeight {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for ScopeVoteWeight {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::ScopeVoteWeight
    }
}

pub fn find_scope_vote_weight_program_address(
    program_id: &Pubkey,
    scope_id: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[scope_id.as_ref(), mint.as_ref()], program_id)
}
pub fn create_scope_vote_weight_program_address_seeds<'a>(
    scope_id: &'a Pubkey,
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    return [scope_id.as_ref(), mint.as_ref(), bump_seed];
}
