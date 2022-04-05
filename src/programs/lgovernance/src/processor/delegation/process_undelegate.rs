//! Program state processor

use crate::{
    error::GovernanceError,
    state::{
        delegation::scope_delegation_record_account::{
            get_scope_delegation_record_data, ScopeDelegationRecordAccount,
        },
        token_owner_budget_record::get_token_owner_budget_record_data_for_token_record,
        vote_power_origin_record::get_vote_power_origin_record_data_for_owner,
        vote_power_owner_record::{
            get_vote_power_owner_record_data,
            get_vote_power_owner_record_data_for_delegation_activity,
        },
    },
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

/// Processes DepositGoverningTokens instruction
pub fn process_undelegate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let delegation_record_info = next_account_info(accounts_iter)?;

    let token_origin_record_info = next_account_info(accounts_iter)?;
    let token_owner_budget_record_info = next_account_info(accounts_iter)?;
    let governing_owner_info = next_account_info(accounts_iter)?;

    let delegatee_vote_power_owner_record_info = next_account_info(accounts_iter)?;
    let delegatee_governing_owner_info = next_account_info(accounts_iter)?;

    let beneficiary_info = next_account_info(accounts_iter)?;
    msg!("X {}", token_origin_record_info.data_is_empty());

    let token_origin_record = get_vote_power_origin_record_data_for_owner(
        program_id,
        token_origin_record_info,
        governing_owner_info,
    )?;
    msg!("XX {}", delegation_record_info.data_is_empty());

    let delegation_record = get_scope_delegation_record_data(
        program_id,
        delegation_record_info,
        &token_origin_record,
        token_origin_record_info,
        governing_owner_info,
        delegatee_vote_power_owner_record_info,
    )?;
    msg!("XXX");
    let mut token_owner_budget_record = get_token_owner_budget_record_data_for_token_record(
        program_id,
        token_owner_budget_record_info,
        &token_origin_record,
        token_origin_record_info,
        governing_owner_info,
    )?;

    let delegatee_token_owner_record =
        get_vote_power_owner_record_data(program_id, delegatee_vote_power_owner_record_info)?;

    // we can only undelegate if delegation is not used actively in any voting,
    if delegation_record.vote_head.is_some()
        || delegation_record.last_vote_head != delegatee_token_owner_record.latest_vote
    {
        return Err(GovernanceError::InvalidDelegatioStateForUndelegation.into());
    }

    /*  if delegatee_token_owner_record.governing_token_mint != token_owner_record.governing_token_mint
    {
        return Err(GovernanceError::InvalidGoverningTokenMint.into());
    } */

    /*
    let scope = delegatee_token_owner_record
        .delegated_by_scope
        .as_ref()
        .unwrap();
    */

    // Update budget
    token_owner_budget_record.amount = token_owner_budget_record
        .amount
        .checked_add(amount)
        .unwrap();

    token_owner_budget_record.serialize(&mut *token_owner_budget_record_info.data.borrow_mut())?;

    // Modify the delegatee token owner record
    let mut delegatee_token_owner_record_data =
        get_vote_power_owner_record_data_for_delegation_activity(
            program_id,
            delegatee_vote_power_owner_record_info,
            delegatee_governing_owner_info.key,
            &token_origin_record.source,
            &delegatee_token_owner_record.delegated_by_scope,
        )?;
    delegatee_token_owner_record_data.amount = delegatee_token_owner_record_data
        .amount
        .checked_sub(amount)
        .unwrap();
    /* match &mut delegatee_token_owner_record_data.source {
        VotePowerSource::Token {
            governing_token_deposit_amount,
            ..
        } => {
            *governing_token_deposit_amount =
                governing_token_deposit_amount.checked_sub(amount).unwrap();
        }
        VotePowerSource::Tag {
            amount: tag_amount, ..
        } => {
            *tag_amount = tag_amount.checked_sub(amount).unwrap();
        }
    }; */

    delegatee_token_owner_record_data
        .serialize(&mut *delegatee_vote_power_owner_record_info.data.borrow_mut())?;

    // Update delegation, might also dispose
    ScopeDelegationRecordAccount::undelegate(
        program_id,
        amount,
        delegation_record_info,
        &token_origin_record,
        token_origin_record_info,
        governing_owner_info,
        &delegatee_token_owner_record,
        delegatee_vote_power_owner_record_info,
        beneficiary_info,
    )?;

    /*  VotePowerOwnerRecord::subtract_amount(
        program_id,
        delegatee_vote_power_owner_record_info,
        delegatee_governing_owner_info.key,
        &token_owner_record.governing_token_mint,
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
