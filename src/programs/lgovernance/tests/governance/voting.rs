use crate::governance::utils::{TestDelegation, TestTagRecordFactory, TestVotePowerSource};
use crate::utils::program_test;
use lgovernance::state::enums::ProposalState;

use lgovernance::state::scopes::scope::VotePowerUnit;
use solana_program_test::*;


use super::super::bench::ProgramTestBench;
use super::utils::{TestGovernance, TestProposal, TestToken, TestUser};

#[tokio::test]
async fn success_vote_execute() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user = TestUser::new();

    let governance_token = TestToken::new(&mut bench).await;

    governance_token
        .create_token_holder_account(&mut bench)
        .await;

    user.create_associated_token_account(&mut bench, &governance_token)
        .await;

    governance_token
        .mint_to(
            &mut bench,
            1,
            &user.get_associated_token_account_address(&governance_token),
        )
        .await;

    user.deposit_governance_tokens(&mut bench, 1, &governance_token)
        .await;

    let vote_power_unit = VotePowerUnit::Mint(governance_token.mint);

    let mut governance = TestGovernance::new(&mut bench).await;
    governance.with_native_treasury(&mut bench).await;

    let transfer_amount = 1;

    let scope = governance
        .create_scope_system(
            &mut bench,
            TestVotePowerSource::TestToken(&governance_token),
        )
        .await;

    // Enable user to create proposal
    user.create_delegatee(&mut bench, &vote_power_unit, &scope)
        .await;

    let self_delegation =
        TestDelegation::new(&mut bench, &user, &user, &vote_power_unit, &scope).await;
    self_delegation.delegate(&mut bench, &1).await;

    let (proposal, recipent_wallet) = TestProposal::new_transfer_proposal(
        &mut bench,
        &user,
        &scope,
        &governance,
        transfer_amount,
    )
    .await;

    let transfer_destination = &recipent_wallet.address;
    let transfer_destination_balance = bench
        .get_account(transfer_destination)
        .await
        .unwrap()
        .lamports;

    // vote for the transaction option
    proposal
        .vote_with_delegate(&mut bench, &vec![1], &user, &vote_power_unit, &scope)
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    );

    proposal
        .advance_clock_past_max_hold_up_time(&mut bench, 1)
        .await;

    proposal.execute_transactions(&mut bench, 1).await;

    assert_eq!(
        bench
            .get_account(transfer_destination)
            .await
            .unwrap()
            .lamports
            - transfer_destination_balance,
        transfer_amount
    )
}

#[tokio::test]
async fn success_token_vote_unvote() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user = TestUser::new();

    let governance_token = TestToken::new(&mut bench).await;

    governance_token
        .create_token_holder_account(&mut bench)
        .await;

    user.create_associated_token_account(&mut bench, &governance_token)
        .await;

    governance_token
        .mint_to(
            &mut bench,
            1,
            &user.get_associated_token_account_address(&governance_token),
        )
        .await;

    user.deposit_governance_tokens(&mut bench, 1, &governance_token)
        .await;

    let vote_power_unit = VotePowerUnit::Mint(governance_token.mint);

    let mut governance = TestGovernance::new(&mut bench).await;
    governance.with_native_treasury(&mut bench).await;

    let scope = governance
        .create_scope_system(
            &mut bench,
            TestVotePowerSource::TestToken(&governance_token),
        )
        .await;

    // Remove temporary authority
    governance
        .update_governance_authority(&mut bench, None)
        .await;

    // Enable user to create proposal
    user.create_delegatee(&mut bench, &vote_power_unit, &scope)
        .await;
    let self_delegation =
        TestDelegation::new(&mut bench, &user, &user, &vote_power_unit, &scope).await;
    self_delegation.delegate(&mut bench, &1).await;

    let (proposal, _recipent_wallet) =
        TestProposal::new_transfer_proposal(&mut bench, &user, &scope, &governance, 1).await;

    // vote for the transaction option
    proposal
        .vote_with_delegate(&mut bench, &vec![0], &user, &vote_power_unit, &scope)
        .await;

    let beneficiary = bench.with_wallet().await;
    let beneficiary_balance = bench
        .get_account(&beneficiary.address)
        .await
        .unwrap()
        .lamports;

    proposal
        .unvote_with_delegate(
            &mut bench,
            vec![0],
            &user,
            &vote_power_unit,
            &scope,
            &beneficiary.address,
        )
        .await;

    // Assert some refund
    assert!(
        bench
            .get_account(&beneficiary.address)
            .await
            .unwrap()
            .lamports
            > beneficiary_balance
    )
}

#[tokio::test]
async fn success_tag_vote_unvote() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user = TestUser::new();

    let tag_record_factory = TestTagRecordFactory::new(&mut bench).await;
    let vote_power_unit = VotePowerUnit::Tag {
        record_factory: tag_record_factory.factory,
    };

    tag_record_factory.new_record(&mut bench, &user).await;

    user.deposit_governance_tag(&mut bench, &tag_record_factory)
        .await;

    let mut governance = TestGovernance::new(&mut bench).await;
    governance.with_native_treasury(&mut bench).await;

    let scope = governance
        .create_scope_system(
            &mut bench,
            TestVotePowerSource::TestTagRecordFactory(&tag_record_factory),
        )
        .await;

    // Enable user to create propsal
    user.create_delegatee(&mut bench, &vote_power_unit, &scope)
        .await;

    let delegation = TestDelegation::new(&mut bench, &user, &user, &vote_power_unit, &scope).await;
    delegation.delegate(&mut bench, &1).await;

    let (proposal, _recipent_wallet) =
        TestProposal::new_transfer_proposal(&mut bench, &user, &scope, &governance, 1).await;

    // vote for the transaction option
    proposal
        .vote_with_delegate(&mut bench, &vec![0], &user, &vote_power_unit, &scope)
        .await;

    let beneficiary = bench.with_wallet().await;
    let beneficiary_balance = bench
        .get_account(&beneficiary.address)
        .await
        .unwrap()
        .lamports;

    proposal
        .unvote_with_delegate(
            &mut bench,
            vec![0],
            &user,
            &vote_power_unit,
            &scope,
            &beneficiary.address,
        )
        .await;

    // Assert some refund
    assert!(
        bench
            .get_account(&beneficiary.address)
            .await
            .unwrap()
            .lamports
            > beneficiary_balance
    )
}
