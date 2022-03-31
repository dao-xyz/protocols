use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        proposal::get_proposal_data,
        scopes::scope::get_scope_data_for_governance,
        token_owner_budget_record::{
            get_token_owner_budget_record_data_for_token_record, TokenOwnerBudgetRecord,
        },
        token_owner_record::get_token_owner_record_data_for_owner,
        vote_record::{
            get_vote_record_address_seeds, get_vote_record_data,
            get_vote_record_data_for_proposal_and_token_owner, VoteRecordV2,
        },
    },
};

use shared::account::create_and_serialize_account_verify_with_bump;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_cast_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account_info = next_account_info(accounts_iter)?;
    let vote_record_info = next_account_info(accounts_iter)?;
    let token_owner_record_info = next_account_info(accounts_iter)?;
    let governing_token_owner_info = next_account_info(accounts_iter)?;
    let scope_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    // TODO: More granular check proposal data?
    let proposal = get_proposal_data(program_id, proposal_account_info)?;
    let scope = get_scope_data_for_governance(program_id, scope_info, &proposal.governance)?;

    msg!("X");
    let mut token_owner_record_data = get_token_owner_record_data_for_owner(
        program_id,
        token_owner_record_info,
        governing_token_owner_info,
    )?;
    msg!("XX");

    let vote_weight = match token_owner_record_data.delegated_by_scope {
        Some(_scope) => {
            /*     let scope_delegation_record_info = next_account_info(accounts_iter)?;
                       let mut scope_delegation_record_data =
                           get_delegation_record_data_for_delegator_and_delegatee(
                               program_id,
                               scope_delegation_record_info,
                               delegator_token_owner_record_info.key,
                               &delegatee_token_owner_record,
                           )?;
            */
            token_owner_record_data.governing_token_deposit_amount
        }
        None => {
            let token_owner_budget_record_info = next_account_info(accounts_iter)?;
            let token_owner_budget_record_data =
                get_token_owner_budget_record_data_for_token_record(
                    program_id,
                    token_owner_budget_record_info,
                    &token_owner_record_data,
                    token_owner_record_info,
                    governing_token_owner_info,
                )?;
            token_owner_budget_record_data.amount
        }
    };

    msg!("XXX");

    // TODO: CHECK OWNER OF POST, CHECK MINTS,
    if !vote_record_info.data_is_empty() {
        return Err(GovernanceError::VoteAlreadyExists.into());
    } else {
        // Update last vote record to link to this new vote
        let last_vote_record_key = if let Some(vote) = token_owner_record_data.latest_vote {
            let last_vote_record_info = next_account_info(accounts_iter)?;
            msg!(
                "LAST VOTE RECORD: {}",
                last_vote_record_info.data_is_empty()
            );
            if &vote != last_vote_record_info.key {
                return Err(GovernanceError::InvalidVoteRecord.into());
            }

            let mut last_vote_data = get_vote_record_data_for_proposal_and_token_owner(
                program_id,
                last_vote_record_info,
                proposal_account_info.key,
                governing_token_owner_info,
            )?;
            msg!("??");
            if last_vote_data.next_vote.is_some() {
                // Expecting head
                return Err(GovernanceError::InvalidVoteRecord.into());
            }
            last_vote_data.next_vote = Some(*vote_record_info.key);
            last_vote_data.serialize(&mut *last_vote_record_info.data.borrow_mut())?;
            Some(*last_vote_record_info.key)
        } else {
            None
        };
        msg!("XXX!");

        let vote = proposal.perform_voting(
            program_id,
            vote_weight,
            true,
            &token_owner_record_data.governing_token_mint,
            scope_info.key,
            &scope,
            proposal_account_info.key,
            accounts_iter,
        )?;

        // Update last vote record to point to the new one

        // Add vote record so we can not vote again through the same scope
        let vote_record_data = VoteRecordV2 {
            account_type: AccountType::VoteRecordV2,
            proposal: *proposal_account_info.key,
            governing_token_owner: *governing_token_owner_info.key,
            vote,
            scope: *scope_info.key,
            is_relinquished: false,
            previous_vote: last_vote_record_key, // move vote in top of the "stack"
            next_vote: None,
        };
        msg!("SAVE!");
        create_and_serialize_account_verify_with_bump::<VoteRecordV2>(
            payer_info,
            vote_record_info,
            &vote_record_data,
            &get_vote_record_address_seeds(
                proposal_account_info.key,
                token_owner_record_info.key,
                scope_info.key,
                &[vote_record_bump_seed],
            ),
            program_id,
            system_info,
            &rent,
        )?;
        msg!("SAVE DONE!");
    }

    // Update TokenOwnerRecord vote counts
    token_owner_record_data.unrelinquished_votes_count = token_owner_record_data
        .unrelinquished_votes_count
        .checked_add(1)
        .unwrap();

    token_owner_record_data.total_votes_count = token_owner_record_data
        .total_votes_count
        .checked_add(1)
        .unwrap();

    token_owner_record_data.latest_vote = Some(*vote_record_info.key);
    if token_owner_record_data.first_vote.is_none() {
        token_owner_record_data.first_vote = Some(*vote_record_info.key);
    }
    // Update propsal
    proposal.serialize(&mut *proposal_account_info.data.borrow_mut())?;
    token_owner_record_data.serialize(&mut *token_owner_record_info.data.borrow_mut())?;

    Ok(())
}
