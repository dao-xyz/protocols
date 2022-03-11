//! Program state processor

use crate::{
    accounts::AccountType,
    state::{
        rule_delegation_account::RuleDelegationAccount,
        token_owner_budget_record::{
            get_token_owner_budget_record_address_seeds, TokenOwnerBudgetRecord,
        },
        token_owner_record::{
            get_token_owner_delegatee_record_address_seeds,
            get_token_owner_record_data_for_seeds, TokenOwnerRecordV2,
        },
    },
    tokens::spl_utils::get_token_balance,
};
use borsh::BorshSerialize;
use shared::account::{create_and_serialize_account_signed, get_account_data};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Processes DepositGoverningTokens instruction
pub fn process_delegate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    rule: &Pubkey,
    delegatee: &Pubkey, // token owner record
    bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governing_token_mint_info = next_account_info(accounts_iter)?;
    let token_owner_record_info = next_account_info(accounts_iter)?;
    let token_owner_budget_record_info = next_account_info(accounts_iter)?;
    let delegation_record_info = next_account_info(accounts_iter)?;
    let delegation_token_owner_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let _spl_token_info = next_account_info(accounts_iter)?;
    let _rent_sysvar_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    // Create/Update budget
    if token_owner_budget_record_info.data_is_empty() {
        let governing_token_holding_info = next_account_info(accounts_iter)?;
        let balance = get_token_balance(governing_token_holding_info)?;
        let balance_after_delegate = balance.checked_sub(amount).unwrap();
        if balance_after_delegate < 0 {
            return Err(ProgramError::InvalidArgument);
        }

        let bump_seeds = [bump_seed];
        let seeds = get_token_owner_budget_record_address_seeds(
            token_owner_record_info.key,
            rule,
            &bump_seeds,
        );
        let rent = Rent::get()?;
        create_and_serialize_account_signed::<TokenOwnerBudgetRecord>(
            payer_info,
            token_owner_budget_record_info,
            &TokenOwnerBudgetRecord {
                account_type: AccountType::TokenOwnerBudgetRecord,
                amount: balance_after_delegate,
                rule: *rule,
            },
            &seeds,
            program_id,
            system_info,
            &rent,
        )?;
    } else {
        let token_owner_budget_record_info = next_account_info(accounts_iter)?;
        let mut token_owner_budget_record =
            get_account_data::<TokenOwnerBudgetRecord>(program_id, token_owner_budget_record_info)?;
        token_owner_budget_record.amount = token_owner_budget_record
            .amount
            .checked_sub(amount)
            .unwrap();

        if token_owner_budget_record.amount < 0 {
            return Err(ProgramError::InvalidArgument);
        }

        token_owner_budget_record
            .serialize(&mut *token_owner_budget_record_info.data.borrow_mut())?;
    }

    // Create delegation record so we can undelegate at some point
    if delegation_record_info.data_is_empty() {
        let bump_seeds = [bump_seed];
        let seeds = get_token_owner_budget_record_address_seeds(
            token_owner_record_info.key,
            rule,
            &bump_seeds,
        );
        let rent = Rent::get()?;

        create_and_serialize_account_signed::<RuleDelegationAccount>(
            payer_info,
            token_owner_budget_record_info,
            &RuleDelegationAccount {
                account_type: AccountType::DelegationRecord,
                amount,
                rule: *rule,
                vote_mint: *governing_token_mint_info.key,
                delegatee: *delegatee,
            },
            &seeds,
            program_id,
            system_info,
            &rent,
        )?;
    } else {
        let rule_delegation_record_info = next_account_info(accounts_iter)?;
        let mut rule_delegation_record =
            get_account_data::<RuleDelegationAccount>(program_id, token_owner_budget_record_info)?;
        rule_delegation_record.amount = rule_delegation_record.amount.checked_add(amount).unwrap();
        rule_delegation_record.serialize(&mut *rule_delegation_record_info.data.borrow_mut())?;
    }

    let delegatee_token_owner_record_address_seeds =
        get_token_owner_delegatee_record_address_seeds(governing_token_mint_info.key, delegatee);

    // Modify the token owner record of the delegatee to take account of the delegation
    if delegation_token_owner_record_info.data_is_empty() {
        let token_owner_record_data = TokenOwnerRecordV2 {
            account_type: AccountType::TokenOwnerRecordV2,
            governing_token_owner: *delegatee,
            governing_token_deposit_amount: amount,
            governing_token_mint: *governing_token_mint_info.key,
            unrelinquished_votes_count: 0,
            total_votes_count: 0,
            delegated_governing_token_deposit_amount: 0,
            outstanding_proposal_count: 0,
            delegated_by_rule: Some(*rule),
        };

        create_and_serialize_account_signed(
            payer_info,
            delegation_token_owner_record_info,
            &token_owner_record_data,
            &delegatee_token_owner_record_address_seeds,
            program_id,
            system_info,
            &rent,
        )?;
    } else {
        let mut token_owner_record_data = get_token_owner_record_data_for_seeds(
            program_id,
            delegation_token_owner_record_info,
            &delegatee_token_owner_record_address_seeds,
        )?;

        token_owner_record_data.governing_token_deposit_amount = token_owner_record_data
            .governing_token_deposit_amount
            .checked_add(amount)
            .unwrap();

        token_owner_record_data.serialize(&mut *token_owner_record_info.data.borrow_mut())?;
    }

    Ok(())
}
