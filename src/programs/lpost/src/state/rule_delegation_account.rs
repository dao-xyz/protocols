use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::MaxSize;
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

use crate::accounts::AccountType;

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct RuleDelegationAccount {
    pub account_type: AccountType,
    pub rule: Pubkey,
    pub vote_mint: Pubkey,
    pub delegatee: Pubkey,
    pub amount: u64,
}

impl MaxSize for RuleDelegationAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}
impl IsInitialized for RuleDelegationAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::TokenOwnerRecordV2
    }
}

pub fn find_rule_delegation_account_program_address(
    program_id: &Pubkey,
    rule: &Pubkey,
    vote_mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[rule.as_ref(), vote_mint.as_ref()], program_id)
}
pub fn create_rule_delegation_account_program_address_seeds<'a>(
    program_id: &Pubkey,
    rule: &'a Pubkey,
    vote_mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    return [rule.as_ref(), vote_mint.as_ref(), bump_seed];
}
