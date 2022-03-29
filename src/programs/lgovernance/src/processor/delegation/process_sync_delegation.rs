use crate::{
    error::GovernanceError,
    state::{
        delegation::rule_delegation_record_account::get_delegation_record_data_for_delegator_and_delegatee,
        proposal::get_proposal_data,
        rules::rule::get_rule_data_for_governance,
        token_owner_record::{get_token_owner_record_data_for_owner, TokenOwnerRecordV2},
        vote_record::{
            get_vote_record_address, get_vote_record_data,
            get_vote_record_data_for_proposal_and_token_owner,
            get_vote_record_data_for_proposal_and_unsigned_token_owner, VoteRecordV2,
        },
    },
};

use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

/// When delegating tokens, you delegate to a "token record"/"user" that might already have a few active votes
/// For this user we might want to update the existing active votes.
/// The opposite case also exist: When we want to undelegate but active votes for a "token record"/"user" exist.
/// Then we have to partially uncast votes, So that the active votes have to correct amount. If we don't do this
/// a user could double spend votes by delegating and undelegating repeatidly
pub fn process_sync_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    delegatee_token_owner_record: Pubkey,
    sync: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account_info = next_account_info(accounts_iter)?;
    let vote_record_info = next_account_info(accounts_iter)?;
    let rule_delegation_record_info = next_account_info(accounts_iter)?;
    let delegator_token_owner_record_info = next_account_info(accounts_iter)?;
    let delegator_governing_token_owner_info = next_account_info(accounts_iter)?;
    let rule_info = next_account_info(accounts_iter)?;

    // TODO: More granular check proposal data?
    let proposal = get_proposal_data(program_id, proposal_account_info)?;
    let rule = get_rule_data_for_governance(program_id, rule_info, &proposal.governance)?;

    // Make sure governing token owner is signer of the delegator token owner record info
    let token_owner_record_data = get_token_owner_record_data_for_owner(
        program_id,
        delegator_token_owner_record_info,
        delegator_governing_token_owner_info,
    )?;

    let mut rule_delegation_record_data = get_delegation_record_data_for_delegator_and_delegatee(
        program_id,
        rule_delegation_record_info,
        delegator_token_owner_record_info.key,
        &delegatee_token_owner_record,
    )?;

    if !vote_record_info.data_is_empty() {
        if &get_vote_record_address(
            program_id,
            proposal_account_info.key,
            &delegatee_token_owner_record,
            rule_info.key,
        )
        .0 != vote_record_info.key
        {
            return Err(GovernanceError::InvalidVoteRecord.into());
        }

        // ----- Update an old vote with delegation amounts

        // Update the casted amount
        if let Some(head) = &rule_delegation_record_data.vote_head {
            if head != vote_record_info.key {
                return Err(GovernanceError::InvalidPreviousVoteForVoteRecord.into());
            }
            let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;

            match sync {
                false => {
                    vote_record_data.assert_vote_equals(&proposal.perform_voting(
                        program_id,
                        rule_delegation_record_data.amount,
                        sync,
                        &token_owner_record_data.governing_token_mint,
                        rule_info.key,
                        &rule,
                        proposal_account_info.key,
                        accounts_iter,
                    )?)?;
                    rule_delegation_record_data.vote_head = vote_record_data.next_vote;
                }
                true => {
                    match &vote_record_data.previous_vote {
                        Some(previous_vote) => {
                            // move back in history
                            let previous_vote_record_info = next_account_info(accounts_iter)?;
                            let previous_proposal_account_info = next_account_info(accounts_iter)?;

                            if previous_vote != previous_vote_record_info.key {
                                return Err(GovernanceError::InvalidVoteRecord.into());
                            }
                            let previous_vote_record_data =
                                get_vote_record_data(program_id, previous_vote_record_info)?;

                            previous_vote_record_data.assert_vote_equals(
                                &proposal.perform_voting(
                                    program_id,
                                    rule_delegation_record_data.amount,
                                    sync,
                                    &token_owner_record_data.governing_token_mint,
                                    rule_info.key,
                                    &rule,
                                    previous_proposal_account_info.key,
                                    accounts_iter,
                                )?,
                            )?;
                            rule_delegation_record_data.vote_head =
                                Some(*previous_vote_record_info.key);
                        }
                        None => {
                            // Nothing more
                            return Err(GovernanceError::InvalidSyncDirection.into());
                        }
                    }
                }
            }
        } else {
            // We can only end up here if the delegation happened before any votes has been casted
            if !sync {
                let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;
                if vote_record_info.key
                    != &get_vote_record_address(
                        program_id,
                        proposal_account_info.key,
                        &delegatee_token_owner_record,
                        rule_info.key,
                    )
                    .0
                {
                    return Err(GovernanceError::InvalidVoteRecord.into());
                }

                if vote_record_data.previous_vote.is_some() {
                    return Err(GovernanceError::InvalidPreviousVoteForVoteRecord.into());
                }

                vote_record_data.assert_vote_equals(&proposal.perform_voting(
                    program_id,
                    rule_delegation_record_data.amount,
                    sync,
                    &token_owner_record_data.governing_token_mint,
                    rule_info.key,
                    &rule,
                    proposal_account_info.key,
                    accounts_iter,
                )?)?;
                rule_delegation_record_data.vote_head = Some(*vote_record_info.key);
            } else {
                // We can not sync history if we have no prior vote records
                return Err(GovernanceError::InvalidSyncDirection.into());
            }
        };

        /*  match &sync {
            true => {
                // Go backward in history (i.e synchronize)
                rule_delegation_record_data.vote_head = vote_record_data.previous_vote;
            }
            false => {
                match &vote_record_data.next_vote {
                    Some(next_vote) => {}
                    None => {}
                };
                // Go forward in history (i.e. unsynchronize/unwind)
                rule_delegation_record_data.vote_head = vote_record_data.next_vote;
            }
        }; */

        // This is a "revote" so lets just check we have not done anything new
    } else {
        return Err(GovernanceError::VoteMissing.into());
    }

    //token_owner_record_data.latest_vote = Some(*vote_record_info.key);

    // Update propsal
    proposal.serialize(&mut *proposal_account_info.data.borrow_mut())?;
    //token_owner_record_data.serialize(&mut *token_owner_record_info.data.borrow_mut())?;

    Ok(())
}
