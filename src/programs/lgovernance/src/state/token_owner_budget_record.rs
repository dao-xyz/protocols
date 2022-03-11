//! Token Owner Consumption Record Account
//! Account for handling budgets that are affected by delegations by rule(s)


use shared::account::{MaxSize};

use crate::{accounts::AccountType};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct TokenOwnerBudgetRecord {
    /// Governance account type
    pub account_type: AccountType,

    /// Governing Token Mint the TokenOwnerRecord holds deposit for
    pub rule: Pubkey,

    /// The budget
    pub amount: u64,
}

impl MaxSize for TokenOwnerBudgetRecord {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for TokenOwnerBudgetRecord {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::TokenOwnerBudgetRecord
    }
}

/// Returns TokenOwnerRecord PDA address
pub fn get_token_owner_budget_record_address(
    program_id: &Pubkey,
    token_owner_record: &Pubkey,
    rule: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(&[token_owner_record.as_ref(), rule.as_ref()], program_id).0
}

/// Returns TokenOwnerRecord PDA seeds
pub fn get_token_owner_budget_record_address_seeds<'a>(
    token_owner_record: &'a Pubkey,
    rule: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [token_owner_record.as_ref(), rule.as_ref(), bump_seed]
}
