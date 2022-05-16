use crate::{
    error::GovernanceError,
    state::{
        delegation::scope_delegation_record_account::get_delegation_record_data_for_delegator_and_delegatee,
        proposal::get_proposal_data, scopes::scope::get_scope_data_for_governance,
        vote_power_origin_record::get_vote_power_origin_record_data_for_owner,
        vote_record::get_vote_record_data,
    },
};
use borsh::BorshSerialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
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
    let proposal_account_info = next_account_info(accounts_iter)?;
    let scope_delegation_record_info = next_account_info(accounts_iter)?;
    let delegator_token_origin_record_info = next_account_info(accounts_iter)?;
    let delegator_governing_owner_info = next_account_info(accounts_iter)?;
    let delegatee_vote_power_owner_record_info = next_account_info(accounts_iter)?;
    let delegatee_governing_owner_info = next_account_info(accounts_iter)?;
    let scope_info = next_account_info(accounts_iter)?;

    let proposal = get_proposal_data(program_id, proposal_account_info)?;

    // Delegator or delegatee has to sign
    if !delegator_governing_owner_info.is_signer && !delegatee_governing_owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut scope_delegation_record_data = get_delegation_record_data_for_delegator_and_delegatee(
        program_id,
        scope_delegation_record_info,
        delegator_token_origin_record_info,
        delegatee_vote_power_owner_record_info,
    )?;
    let delegator_token_origin_record_data = get_vote_power_origin_record_data_for_owner(
        program_id,
        delegator_token_origin_record_info,
        delegator_governing_owner_info,
    )?;

    let scope = get_scope_data_for_governance(program_id, scope_info, &proposal.governance)?;

    // Get delegatee info token owner record info
    /*  let delegatee_vote_power_owner_record_info: &AccountInfo =
    match delegator_or_delegatee_vote_power_owner_record_info.key
        == &scope_delegation_record_data.delegatee_token_owner_record
    {
        true => Ok(delegator_or_delegatee_vote_power_owner_record_info),
        false => {
            let delegatee_vote_power_owner_record_info = next_account_info(accounts_iter)?;
            if delegatee_vote_power_owner_record_info.key
                != &scope_delegation_record_data.delegatee_token_owner_record
            {
                Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into())
            } else {
                Ok(delegatee_vote_power_owner_record_info)
            }
        }
    }
    .unwrap(); */

    // Make sure governing token owner is signer of the delegator token owner record info

    if !vote_record_info.data_is_empty() {
        // ----- Update an old vote with delegation amounts

        // Update the casted amount
        let vote_record_data = get_vote_record_data(program_id, vote_record_info)?;

        /*       if let Some(head) = &scope_delegation_record_data.vote_head {
            let next_vote_record_info = next_account_info(accounts_iter)?;

            if head != next_vote_record_info.key {
                return Err(GovernanceError::InvalidPreviousVoteForVoteRecord.into());
            }
            let next_vote_record_data = get_vote_record_data(program_id, next_vote_record_info)?;

            match &next_vote_record_data.previous_vote {
                Some(previous_vote) => {
                    // move back in history
                    if previous_vote != vote_record_info.key {
                        return Err(GovernanceError::InvalidVoteRecord.into());
                    }
                }
                None => {
                    // Nothing more
                    return Err(GovernanceError::InvalidSyncDirection.into());
                }
            }
        } else  */
        if let Some(head) = &scope_delegation_record_data.last_vote_head {
            if head != vote_record_info.key {
                return Err(GovernanceError::InvalidPreviousVoteForVoteRecord.into());
            }
        } else {
            // We can only end up here if the delegation happened before any votes has been casted
            // We can not sync history if we have no prior vote records
            return Err(GovernanceError::InvalidSyncDirection.into());
        };

        vote_record_data.assert_vote_equals(&proposal.perform_voting(
            program_id,
            scope_delegation_record_data.amount,
            true,
            &delegator_token_origin_record_data.source,
            scope_info.key,
            &scope,
            proposal_account_info.key,
            accounts_iter,
        )?)?;
        scope_delegation_record_data.last_vote_head = scope_delegation_record_data.vote_head;
        scope_delegation_record_data.vote_head = Some(*vote_record_info.key);
        scope_delegation_record_data
            .serialize(&mut *scope_delegation_record_info.data.borrow_mut())?;
        proposal.serialize(&mut *proposal_account_info.data.borrow_mut())?;
    } else {
        return Err(GovernanceError::VoteMissing.into());
    }

    Ok(())
}
