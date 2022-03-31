use crate::{
    error::GovernanceError,
    state::{
        enums::ProposalState,
        proposal::get_proposal_data,
        scopes::scope::get_scope_data_for_governance,
        token_owner_record::get_token_owner_record_data_for_owner,
        vote_record::{get_vote_record_data_for_proposal_and_token_owner, Vote, VoteRecordV2},
    },
};
use shared::account::{dispose_account, get_account_data, MaxSize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_uncast_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_info = next_account_info(accounts_iter)?;
    let vote_record_info = next_account_info(accounts_iter)?;
    let token_owner_record_info = next_account_info(accounts_iter)?;
    let governing_token_owner_record_info = next_account_info(accounts_iter)?;
    let scope_info = next_account_info(accounts_iter)?;
    let proposal = get_proposal_data(program_id, proposal_info)?;
    let scope = get_scope_data_for_governance(program_id, scope_info, &proposal.governance)?;

    let mut token_owner_record_data = get_token_owner_record_data_for_owner(
        program_id,
        token_owner_record_info,
        governing_token_owner_record_info,
    )?;

    let mut vote_record_data = get_vote_record_data_for_proposal_and_token_owner(
        program_id,
        vote_record_info,
        proposal_info.key,
        governing_token_owner_record_info,
    )?;

    vote_record_data.assert_can_relinquish_vote()?;

    let clock = Clock::get()?;

    // If the Proposal is still being voted on then the token owner vote will be withdrawn and it won't count towards the vote outcome
    // Note: If there is no tipping point the proposal can be still in Voting state but already past the configured max_voting_time
    //       It means it awaits manual finalization (FinalizeVote) and it should no longer be possible to withdraw the vote and we only release the tokens
    if proposal.state == ProposalState::Voting
        && !proposal.has_vote_time_ended(&scope.config.time_config, clock.unix_timestamp)
    {
        msg!("A");
        let beneficiary_info = next_account_info(accounts_iter)?;
        msg!("AA {} ", beneficiary_info.key);

        // Note: It's only required to sign by governing_authority if relinquishing the vote results in vote change
        // If the Proposal is already decided then anybody can prune active votes for token owner
        token_owner_record_data
            .assert_token_owner_or_delegate_is_signer(governing_token_owner_record_info)?;
        msg!("AAA");

        proposal.perform_voting(
            program_id,
            token_owner_record_data.governing_token_deposit_amount,
            false,
            &token_owner_record_data.governing_token_mint,
            scope_info.key,
            &scope,
            proposal_info.key,
            accounts_iter,
        )?;
        msg!("AAAA");

        proposal.serialize(&mut *proposal_info.data.borrow_mut())?;
        msg!("AAAAA");

        dispose_account(vote_record_info, beneficiary_info);

        // Connect adjacent votes (TODO cleanup this mess)
        let previous = if let Some(previous_vote) = vote_record_data.previous_vote {
            let vote_record_info = next_account_info(accounts_iter)?;
            if &previous_vote != vote_record_info.key {
                return Err(GovernanceError::InvalidVoteRecord.into());
            }
            let vote_record_data = get_account_data::<VoteRecordV2>(program_id, vote_record_info)?;
            Some((vote_record_info, vote_record_data))
        } else {
            None
        };

        let next = if let Some(next_vote) = vote_record_data.next_vote {
            let vote_record_info = next_account_info(accounts_iter)?;
            if &next_vote != vote_record_info.key {
                return Err(GovernanceError::InvalidVoteRecord.into());
            }
            let vote_record_data = get_account_data::<VoteRecordV2>(program_id, vote_record_info)?;
            Some((vote_record_info, vote_record_data))
        } else {
            None
        };

        if previous.is_some() && next.is_some() {
            let mut previous = previous.unwrap();
            let mut next = next.unwrap();
            previous.1.next_vote = Some(*next.0.key);
            previous.1.serialize(&mut *previous.0.data.borrow_mut())?;
            next.1.previous_vote = Some(*previous.0.key);
            next.1.serialize(&mut *next.0.data.borrow_mut())?;
        } else if previous.is_some() {
            let mut previous = previous.unwrap();
            previous.1.next_vote = None;
            previous.1.serialize(&mut *previous.0.data.borrow_mut())?;
        } else if next.is_some() {
            let mut next = next.unwrap();
            next.1.previous_vote = None;
            next.1.serialize(&mut *next.0.data.borrow_mut())?;
            token_owner_record_data.first_vote = Some(*next.0.key);
        } else {
            // None, None
            token_owner_record_data.first_vote = None;
        }

        // Update total votes counts
        token_owner_record_data.total_votes_count = token_owner_record_data
            .total_votes_count
            .checked_sub(1)
            .unwrap();
    } else {
        vote_record_data.is_relinquished = true;
        vote_record_data.serialize(&mut *vote_record_info.data.borrow_mut())?;
    }

    // If the Proposal has been already voted on then we only have to decrease unrelinquished_votes_count
    token_owner_record_data.unrelinquished_votes_count = token_owner_record_data
        .unrelinquished_votes_count
        .checked_sub(1)
        .unwrap();

    token_owner_record_data.serialize(&mut *token_owner_record_info.data.borrow_mut())?;
    msg!("XXXXX");

    Ok(())
}
