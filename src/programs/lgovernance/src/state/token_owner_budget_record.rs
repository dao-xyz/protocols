//! Token Owner Consumption Record Account
//! Account for handling budgets that are affected by delegations by scope(s)

use shared::account::{get_account_data, MaxSize};

use crate::{accounts::AccountType, error::GovernanceError};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

use super::vote_power_origin_record::VotePowerOriginRecord;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct TokenOwnerBudgetRecord {
    /// Governance account type
    pub account_type: AccountType,

    /// Budget for token owner record
    pub token_owner_record: Pubkey,

    /// Budget for scope
    pub scope: Pubkey,

    /// The budget
    pub amount: u64,
}

impl TokenOwnerBudgetRecord {
    /*   pub fn spend<'a>(
    program_id: &Pubkey,
    amount: u64,
    token_owner_budget_record: &mut TokenOwnerBudgetRecord,
    token_owner_budget_record_info: &AccountInfo<'a>, */
    /*  token_owner_record: &VotePowerOwnerRecord,
    vote_power_owner_record_info: &AccountInfo<'a>,
    governing_owner_info: &AccountInfo<'a>, */
    /*    ) -> Result<(), ProgramError> {
        token_owner_budget_record.amount = token_owner_budget_record
            .amount
            .checked_sub(amount)
            .unwrap();

        if token_owner_budget_record.amount < 0 {
            return Err(ProgramError::InvalidArgument);
        }

        token_owner_budget_record
            .serialize(&mut *token_owner_budget_record_info.data.borrow_mut())?;

        Ok(())
    } */

    /*   pub fn unspend<'a>(
    program_id: &Pubkey,
    amount: u64,
    token_owner_budget_record: &mut TokenOwnerBudgetRecord, */
    /* token_owner_budget_record_info: &AccountInfo<'a>,
    token_owner_record: &VotePowerOwnerRecord,
     vote_power_owner_record_info: &AccountInfo<'a>,
     governing_owner_info: &AccountInfo<'a>, */
    /*     ) -> Result<(), ProgramError> {
        token_owner_budget_record.amount = token_owner_budget_record
            .amount
            .checked_add(amount)
            .unwrap();

        token_owner_budget_record
            .serialize(&mut *token_owner_budget_record_info.data.borrow_mut())?;
        Ok(())
    } */
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
/*
/// Returns TokenOwnerRecord PDA address
pub fn assert_token_owner_budget_record_address<'a>(
    program_id: &'a Pubkey,
    token_owner_record: &'a Pubkey,
    scope: &'a Pubkey,
    bump_seed: &'a [u8],
    account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let address = Pubkey::create_program_address(
        &get_token_owner_budget_record_address_seeds(token_owner_record, scope, bump_seed),
        program_id,
    )?;
    if &address != account_info.key {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }
    Ok(())
}
 */
/// Returns TokenOwnerRecord PDA address
pub fn get_token_owner_budget_record_address(
    program_id: &Pubkey,
    token_origin_record: &Pubkey,
    scope: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"budget_record",
            token_origin_record.as_ref(),
            scope.as_ref(),
        ],
        program_id,
    )
}

/// Returns TokenOwnerRecord PDA seeds
pub fn get_token_owner_budget_record_address_seeds<'a>(
    token_origin_record: &'a Pubkey,
    scope: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        b"budget_record",
        token_origin_record.as_ref(),
        scope.as_ref(),
        bump_seed,
    ]
}

/// Deserializes TokenOwneBudgetRecord account and asserts it belongs to the given realm
pub fn get_token_owner_budget_record_data_for_token_record(
    program_id: &Pubkey,
    token_owner_budget_record_info: &AccountInfo,
    token_origin_record: &VotePowerOriginRecord,
    token_origin_record_info: &AccountInfo,
    governing_owner_info: &AccountInfo,
) -> Result<TokenOwnerBudgetRecord, ProgramError> {
    if !governing_owner_info.is_signer {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }
    if &token_origin_record.governing_owner != governing_owner_info.key {
        return Err(GovernanceError::InvalidTokenOwner.into());
    }

    let token_owner_budget_record =
        get_account_data::<TokenOwnerBudgetRecord>(program_id, token_owner_budget_record_info)?;

    if &token_owner_budget_record.token_owner_record != token_origin_record_info.key {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }

    Ok(token_owner_budget_record)
}
