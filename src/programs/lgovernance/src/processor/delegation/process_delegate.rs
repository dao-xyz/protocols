//! Program state processor

use crate::state::{
    delegation::scope_delegation_record_account::ScopeDelegationRecordAccount,
    token_owner_budget_record::get_token_owner_budget_record_data_for_token_record,
    vote_power_origin_record::get_vote_power_origin_record_data_for_owner,
    vote_power_owner_record::get_vote_power_owner_record_data_for_delegation_activity,
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Processes Delegate instruction
pub fn process_delegate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    delegation_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let delegation_record_info = next_account_info(accounts_iter)?;

    let token_origin_record_info = next_account_info(accounts_iter)?;
    let token_owner_budget_record_info = next_account_info(accounts_iter)?;
    let governing_owner_info = next_account_info(accounts_iter)?;

    let delegatee_vote_power_owner_record_info = next_account_info(accounts_iter)?;
    let delegatee_governing_owner_info = next_account_info(accounts_iter)?;

    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    // Load token owner record
    let token_origin_record = get_vote_power_origin_record_data_for_owner(
        program_id,
        token_origin_record_info,
        governing_owner_info,
    )?;

    /*
    if token_owner_record.delegated_by_scope.is_some() {
        return Err(GovernanceError::DelegatingDelegateNotAllowed.into());
    }

    let governing_token_mint = &token_owner_record.governing_token_mint;
    */
    // Update budget
    let mut token_owner_budget_record = get_token_owner_budget_record_data_for_token_record(
        program_id,
        token_owner_budget_record_info,
        &token_origin_record,
        token_origin_record_info,
        governing_owner_info,
    )?;
    let scope = &token_owner_budget_record.scope;

    token_owner_budget_record.amount = token_owner_budget_record
        .amount
        .checked_sub(amount)
        .unwrap();

    token_owner_budget_record.serialize(&mut *token_owner_budget_record_info.data.borrow_mut())?;

    // Load delegatee token owner record
    let mut delegatee_token_owner_record_data =
        get_vote_power_owner_record_data_for_delegation_activity(
            program_id,
            delegatee_vote_power_owner_record_info,
            delegatee_governing_owner_info.key,
            &token_origin_record.source,
            scope,
        )?;

    // Modify the delegatee token owner record
    /*  match &mut delegatee_token_owner_record_data.source {
        VoteSource::Token {
            governing_token_deposit_amount,
            ..
        } => {
            *governing_token_deposit_amount =
                governing_token_deposit_amount.checked_add(amount).unwrap();
        }
        VoteSource::Tag {
            amount: tag_amount, ..
        } => {
            *tag_amount = tag_amount.checked_add(amount).unwrap();
        }
    }; */
    delegatee_token_owner_record_data.amount = delegatee_token_owner_record_data
        .amount
        .checked_add(amount)
        .unwrap();

    // Create delegation record so we can undelegate at some point
    ScopeDelegationRecordAccount::delegate(
        program_id,
        amount,
        scope,
        &rent,
        delegation_record_info,
        delegation_record_bump_seed,
        &token_origin_record,
        token_origin_record_info,
        governing_owner_info,
        &delegatee_token_owner_record_data,
        delegatee_vote_power_owner_record_info,
        payer_info,
        system_info,
    )?;

    delegatee_token_owner_record_data
        .serialize(&mut *delegatee_vote_power_owner_record_info.data.borrow_mut())?;

    /*
    VotePowerOwnerRecord::add_amount(
        program_id,
        delegatee_vote_power_owner_record_info,
        delegatee_governing_owner_info.key,
        &governing_token_mint,
        Some(scope),
        amount,
    )?; */
    // for all outstanding active votes, done by the delegatee_token_owner_record,
    // update cast vote info
    /* if delegatee_vote_power_owner_record_info.data_is_empty() {
           let token_owner_record_data = VotePowerOwnerRecord {
               account_type: AccountType::VotePowerOwnerRecord,
               governing_owner: delegatee.clone(),
               governing_token_deposit_amount: amount,
               governing_token_mint: *governing_token_mint_info.key,
               unrelinquished_votes_count: 0,
               total_votes_count: 0,
               delegated_governing_token_deposit_amount: 0,
               outstanding_proposal_count: 0,
               delegated_by_scope: Some(scope),
           };

           create_and_serialize_account_signed(
               payer_info,
               delegatee_vote_power_owner_record_info,
               &token_owner_record_data,
               &delegatee_vote_power_owner_record_address_seeds,
               program_id,
               system_info,
               &rent,
           )?;
       } else {
           let mut token_owner_record_data = get_vote_power_owner_record_data_for_seeds(
               program_id,
               delegatee_vote_power_owner_record_info,
               &delegatee_vote_power_owner_record_address_seeds,
           )?;

           token_owner_record_data.governing_token_deposit_amount = token_owner_record_data
               .governing_token_deposit_amount
               .checked_add(amount)
               .unwrap();

           token_owner_record_data.serialize(&mut *vote_power_owner_record_info.data.borrow_mut())?;
       }
    */
    Ok(())
}
