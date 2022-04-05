use crate::governance::utils::TestTagRecordFactory;
use crate::{governance::utils::TestDelegation, utils::program_test};
use lgovernance::state::enums::ProposalState;

use lgovernance::state::scopes::scope::VotePowerUnit;
use solana_program_test::*;

use super::super::bench::ProgramTestBench;
use super::utils::{TestGovernance, TestProposal, TestToken, TestUser, TestVotePowerSource};

#[tokio::test]
async fn success_token_vote_after_delegate() {
    // Delegate before vote cast
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user_delegatee = TestUser::new();
    let user_delegator = TestUser::new();

    let governance_token = TestToken::new(&mut bench).await;
    governance_token
        .create_token_holder_account(&mut bench)
        .await;

    // Provide governance tokens to accounts
    user_delegatee
        .create_associated_token_account(&mut bench, &governance_token)
        .await;

    governance_token
        .mint_to(
            &mut bench,
            1,
            &user_delegatee.get_associated_token_account_address(&governance_token),
        )
        .await;

    user_delegatee
        .deposit_governance_tokens(&mut bench, 1, &governance_token)
        .await;

    user_delegator
        .create_associated_token_account(&mut bench, &governance_token)
        .await;

    governance_token
        .mint_to(
            &mut bench,
            99,
            &user_delegator.get_associated_token_account_address(&governance_token),
        )
        .await;

    user_delegator
        .deposit_governance_tokens(&mut bench, 99, &governance_token)
        .await;

    let mut governance = TestGovernance::new(&mut bench).await;
    governance.with_native_treasury(&mut bench).await;

    let scope = governance
        .create_scope_system(
            &mut bench,
            TestVotePowerSource::TestToken(&governance_token),
        )
        .await;

    let vote_power_unit = VotePowerUnit::Mint(governance_token.mint);

    // Enable delegatee to vote with owned tokens
    let self_delegation = TestDelegation::new(
        &mut bench,
        &user_delegatee,
        &user_delegatee,
        &vote_power_unit,
        &scope,
    )
    .await;
    self_delegation.delegate(&mut bench, &1).await;

    let (proposal, _destination) =
        TestProposal::new_transfer_proposal(&mut bench, &user_delegatee, &scope, &governance, 1)
            .await;

    // Enable delegation
    let delegation = TestDelegation::new(
        &mut bench,
        &user_delegator,
        &user_delegatee,
        &vote_power_unit,
        &scope,
    )
    .await;

    // Delegate so that the delegatee now controls 51% of supply
    delegation.delegate(&mut bench, &50).await;

    // vote for the transaction option
    proposal
        .vote_with_delegate(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &vote_power_unit,
            &scope,
        )
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    ); // Enough votes now since delegated

    delegation.undelegate_history(&mut bench).await;

    delegation.undelegate(&mut bench, &49).await;
}

#[tokio::test]
async fn success_token_delegate_undelegate_with_synchronization() {
    // Delegate when vote already cast.
    // Call synchronization funcionality to update previously casted vote
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user_delegatee = TestUser::new();
    let user_delegator = TestUser::new();

    let governance_token = TestToken::new(&mut bench).await;
    governance_token
        .create_token_holder_account(&mut bench)
        .await;

    // Provide governance tokens to accounts
    user_delegatee
        .create_associated_token_account(&mut bench, &governance_token)
        .await;

    governance_token
        .mint_to(
            &mut bench,
            1,
            &user_delegatee.get_associated_token_account_address(&governance_token),
        )
        .await;

    user_delegatee
        .deposit_governance_tokens(&mut bench, 1, &governance_token)
        .await;

    user_delegator
        .create_associated_token_account(&mut bench, &governance_token)
        .await;

    governance_token
        .mint_to(
            &mut bench,
            99,
            &user_delegator.get_associated_token_account_address(&governance_token),
        )
        .await;

    user_delegator
        .deposit_governance_tokens(&mut bench, 99, &governance_token)
        .await;

    let mut governance = TestGovernance::new(&mut bench).await;
    governance.with_native_treasury(&mut bench).await;

    let scope = governance
        .create_scope_system(
            &mut bench,
            TestVotePowerSource::TestToken(&governance_token),
        )
        .await;
    let vote_power_unit = VotePowerUnit::Mint(governance_token.mint);

    // Enable some voting from the delegatee owned tokens
    let self_delegation = TestDelegation::new(
        &mut bench,
        &user_delegatee,
        &user_delegatee,
        &vote_power_unit,
        &scope,
    )
    .await;
    self_delegation.delegate(&mut bench, &1).await;

    let (proposal, _destination) =
        TestProposal::new_transfer_proposal(&mut bench, &user_delegatee, &scope, &governance, 1)
            .await;

    // vote for the transaction option
    proposal
        .vote_with_delegate(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &vote_power_unit,
            &scope,
        )
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(proposal.get_state(&mut bench).await, ProposalState::Voting); // Not enought votes

    let delegation = TestDelegation::new(
        &mut bench,
        &user_delegator,
        &user_delegatee,
        &vote_power_unit,
        &scope,
    )
    .await;

    // Delegate so that the delegatee now controls 51% of supply
    delegation.delegate(&mut bench, &50).await;
    delegation.delegate_history(&mut bench).await;
    let delegatee_token_owner_record = delegation
        .get_delegatee_token_owner_record(&mut bench)
        .await
        .unwrap();

    assert_eq!(delegatee_token_owner_record.amount, 51); // Delegatee + Delegator
    assert_eq!(
        delegation
            .get_delegation_record(&mut bench)
            .await
            .unwrap()
            .amount,
        50
    );

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    ); // Enough votes now since delegated

    delegation.undelegate_history(&mut bench).await;

    delegation.undelegate(&mut bench, &49).await;

    assert_eq!(
        delegation
            .get_delegation_record(&mut bench)
            .await
            .unwrap()
            .amount,
        1
    );
    assert_eq!(
        delegation
            .get_delegatee_token_owner_record(&mut bench)
            .await
            .unwrap()
            .amount,
        2
    );

    delegation.undelegate(&mut bench, &1).await;

    // Delegation record should not exist anymore, since we autmatically dispose if no more tokens left
    assert!(delegation.get_delegation_record(&mut bench).await.is_none());

    assert_eq!(
        delegation
            .get_delegatee_token_owner_record(&mut bench)
            .await
            .unwrap()
            .amount,
        1
    );
}

#[tokio::test]
async fn success_tag_delegate_undelegate_with_synchronization() {
    // Delegate when vote already cast.
    // Call synchronization funcionality to update previously casted vote
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user_delegatee = TestUser::new();
    let user_delegator = TestUser::new();

    let tag_record_factory = TestTagRecordFactory::new(&mut bench).await;
    let vote_power_unit = VotePowerUnit::Tag {
        record_factory: tag_record_factory.factory,
    };

    tag_record_factory
        .new_record(&mut bench, &user_delegatee)
        .await;

    user_delegatee
        .deposit_governance_tag(&mut bench, &tag_record_factory)
        .await;

    tag_record_factory
        .new_record(&mut bench, &user_delegator)
        .await;

    user_delegator
        .deposit_governance_tag(&mut bench, &tag_record_factory)
        .await;

    let mut governance = TestGovernance::new(&mut bench).await;
    governance.with_native_treasury(&mut bench).await;

    let scope = governance
        .create_scope_system(
            &mut bench,
            TestVotePowerSource::TestTagRecordFactory(&tag_record_factory),
        )
        .await;

    // Enable some voting from the delegatee owned tokens
    let self_delegation = TestDelegation::new(
        &mut bench,
        &user_delegatee,
        &user_delegatee,
        &vote_power_unit,
        &scope,
    )
    .await;
    self_delegation.delegate(&mut bench, &1).await;

    let (proposal, _destination) =
        TestProposal::new_transfer_proposal(&mut bench, &user_delegatee, &scope, &governance, 1)
            .await;

    // vote for the transaction option
    proposal
        .vote_with_delegate(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &vote_power_unit,
            &scope,
        )
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(proposal.get_state(&mut bench).await, ProposalState::Voting); // Not enought votes

    let delegation = TestDelegation::new(
        &mut bench,
        &user_delegator,
        &user_delegatee,
        &vote_power_unit,
        &scope,
    )
    .await;

    // Delegate so that the delegatee now controls 51% of supply
    delegation.delegate(&mut bench, &1).await;
    delegation.delegate_history(&mut bench).await;
    let delegatee_token_owner_record = delegation
        .get_delegatee_token_owner_record(&mut bench)
        .await
        .unwrap();

    assert_eq!(delegatee_token_owner_record.amount, 2); // Delegatee + Delegator
    assert_eq!(
        delegation
            .get_delegation_record(&mut bench)
            .await
            .unwrap()
            .amount,
        1
    );

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    ); // Enough votes now since delegated

    delegation.undelegate_history(&mut bench).await;

    delegation.undelegate(&mut bench, &1).await;

    // Delegation record should not exist anymore, since we autmatically dispose if no more tokens left
    assert!(delegation.get_delegation_record(&mut bench).await.is_none());

    assert_eq!(
        delegation
            .get_delegatee_token_owner_record(&mut bench)
            .await
            .unwrap()
            .amount,
        1
    );
}
