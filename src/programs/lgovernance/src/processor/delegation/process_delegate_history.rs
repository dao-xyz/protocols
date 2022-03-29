use crate::{
    error::GovernanceError,
    state::{
        delegation::rule_delegation_record_account::get_delegation_record_data_for_delegator,
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
use borsh::BorshSerialize;

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
pub fn process_delegate_history(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vote_record_info = next_account_info(accounts_iter)?;
    let previous_vote_record_info = next_account_info(accounts_iter)?;
    let previous_proposal_account_info = next_account_info(accounts_iter)?;
    let rule_delegation_record_info = next_account_info(accounts_iter)?;
    let delegator_token_owner_record_info = next_account_info(accounts_iter)?;
    let delegator_governing_token_owner_info = next_account_info(accounts_iter)?;
    let rule_info = next_account_info(accounts_iter)?;

    // TODO: More granular check proposal data?
    let previous_proposal = get_proposal_data(program_id, previous_proposal_account_info)?;

    // Make sure governing token owner is signer of the delegator token owner record info
    let token_owner_record_data = get_token_owner_record_data_for_owner(
        program_id,
        delegator_token_owner_record_info,
        delegator_governing_token_owner_info,
    )?;

    let mut rule_delegation_record_data = get_delegation_record_data_for_delegator(
        program_id,
        rule_delegation_record_info,
        delegator_token_owner_record_info.key,
    )?;
    let delegatee_token_owner_record = &rule_delegation_record_data.delegatee_token_owner_record;

    if !vote_record_info.data_is_empty() {
        // ----- Update an old vote with delegation amounts

        // Update the casted amount
        if let Some(head) = &rule_delegation_record_data.vote_head {
            if head != vote_record_info.key {
                return Err(GovernanceError::InvalidPreviousVoteForVoteRecord.into());
            }
            let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;

            match &vote_record_data.previous_vote {
                Some(previous_vote) => {
                    // move back in history

                    if previous_vote != previous_vote_record_info.key {
                        return Err(GovernanceError::InvalidVoteRecord.into());
                    }
                    let previous_vote_record_data =
                        get_vote_record_data(program_id, previous_vote_record_info)?;

                    let rule = get_rule_data_for_governance(
                        program_id,
                        rule_info,
                        &previous_proposal.governance,
                    )?;

                    previous_vote_record_data.assert_vote_equals(
                        &previous_proposal.perform_voting(
                            program_id,
                            rule_delegation_record_data.amount,
                            true,
                            &token_owner_record_data.governing_token_mint,
                            rule_info.key,
                            &rule,
                            previous_proposal_account_info.key,
                            accounts_iter,
                        )?,
                    )?;
                    rule_delegation_record_data.vote_head = Some(*previous_vote_record_info.key);
                    rule_delegation_record_data
                        .serialize(&mut *rule_delegation_record_info.data.borrow_mut())?;
                    previous_proposal
                        .serialize(&mut *previous_proposal_account_info.data.borrow_mut())?;
                }
                None => {
                    // Nothing more
                    return Err(GovernanceError::InvalidSyncDirection.into());
                }
            }
        } else {
            // We can only end up here if the delegation happened before any votes has been casted
            // We can not sync history if we have no prior vote records
            return Err(GovernanceError::InvalidSyncDirection.into());
        };
    } else {
        return Err(GovernanceError::VoteMissing.into());
    }

    Ok(())
}
