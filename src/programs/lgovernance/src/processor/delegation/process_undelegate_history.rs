use crate::{
    error::GovernanceError,
    state::{
        delegation::rule_delegation_record_account::get_delegation_record_data_for_delegator_or_delegatee,
        proposal::get_proposal_data,
        rules::rule::get_rule_data_for_governance,
        token_owner_record::get_token_owner_record_data_for_owner,
        vote_record::{get_vote_record_address, get_vote_record_data},
    },
};

use borsh::BorshSerialize;
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
pub fn process_undelegate_history(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vote_record_info = next_account_info(accounts_iter)?;
    let proposal_account_info = next_account_info(accounts_iter)?;
    let rule_delegation_record_info = next_account_info(accounts_iter)?;
    let delegator_or_delegatee_token_owner_record_info = next_account_info(accounts_iter)?;
    let delegator_or_delegatee_governing_token_owner_info = next_account_info(accounts_iter)?;
    let rule_info = next_account_info(accounts_iter)?;

    // TODO: More granular check proposal data?
    let proposal = get_proposal_data(program_id, proposal_account_info)?;
    let rule = get_rule_data_for_governance(program_id, rule_info, &proposal.governance)?;

    /*
     */
    /*  let delegatee_token_owner_record_info: &AccountInfo =
    match delegator_or_delegatee_token_owner_record_info.key
        == &rule_delegation_record_data.delegatee_token_owner_record
    {
        true => Ok(delegator_or_delegatee_token_owner_record_info),
        false => {
            let delegatee_token_owner_record_info = next_account_info(accounts_iter)?;
            if delegatee_token_owner_record_info.key
                != &rule_delegation_record_data.delegatee_token_owner_record
            {
                Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into())
            } else {
                Ok(delegatee_token_owner_record_info)
            }
        }
    }
    .unwrap(); */

    // Make sure governing token owner is signer of the delegator token owner record info
    msg!("?");

    let mut rule_delegation_record_data = get_delegation_record_data_for_delegator_or_delegatee(
        program_id,
        rule_delegation_record_info,
        delegator_or_delegatee_token_owner_record_info,
    )?;

    msg!("??");
    let token_owner_record_data = get_token_owner_record_data_for_owner(
        program_id,
        delegator_or_delegatee_token_owner_record_info,
        delegator_or_delegatee_governing_token_owner_info,
    )?;
    // let delegatee_token_owner_record = &rule_delegation_record_data.delegatee_token_owner_record;

    if !vote_record_info.data_is_empty() {
        // ----- Update an old vote with delegation amounts

        // Check that we are updating the right vote
        let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;
        if let Some(head) = &rule_delegation_record_data.vote_head {
            if head != vote_record_info.key {
                return Err(GovernanceError::InvalidVoteRecord.into());
            }
        } else if let Some(head) = &rule_delegation_record_data.last_vote_head {
            // We expect that the vote head is 1 step ahead of the "created_at_vote"
            let previous_vote_info = next_account_info(accounts_iter)?;
            let previous_vote_data = get_vote_record_data(program_id, previous_vote_info)?;
            if head != previous_vote_data.next_vote.as_ref().unwrap() {
                msg!("A");
                return Err(GovernanceError::InvalidVoteRecord.into());
            }
        } else {
            // The delegation happen beofre any casted votes, created_at_vote = NONE and vote_head = NONE
            if &get_vote_record_address(
                program_id,
                proposal_account_info.key,
                &rule_delegation_record_data.delegatee_token_owner_record,
                rule_info.key,
            )
            .0 != vote_record_info.key
            {
                msg!("B");

                return Err(GovernanceError::InvalidVoteRecord.into());
            }
            let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;
            if vote_record_data.previous_vote.is_some() {
                // Not the first vote!
                return Err(GovernanceError::InvalidPreviousVoteForVoteRecord.into());
            }
        }

        msg!("ZZZZ");

        // Update the casted amount
        vote_record_data.assert_vote_equals(&proposal.perform_voting(
            program_id,
            rule_delegation_record_data.amount,
            false,
            &token_owner_record_data.governing_token_mint,
            rule_info.key,
            &rule,
            proposal_account_info.key,
            accounts_iter,
        )?)?;
        rule_delegation_record_data.last_vote_head = Some(*vote_record_info.key);
        rule_delegation_record_data.vote_head = vote_record_data.next_vote;
        rule_delegation_record_data
            .serialize(&mut *rule_delegation_record_info.data.borrow_mut())?;
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
