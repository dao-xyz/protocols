use crate::utils::program_test;
use lgovernance::{
    instruction::CreateProposalOptionType,
    state::{
        enums::ProposalState,
        native_treasury::get_native_treasury_address,
        proposal::{
            proposal_option::ProposalOption, proposal_transaction::ConditionedInstruction, VoteType,
        },
        rules::rule::{RuleCondition, RuleConfig},
    },
};
use solana_program::{borsh::try_from_slice_unchecked, system_instruction, system_program};
use solana_program_test::*;
use solana_sdk::signature::Keypair;

use super::super::bench::ProgramTestBench;
use super::utils::{TestChannel, TestGovernance, TestProposal, TestToken, TestUser};

#[tokio::test]
async fn success_delegate_simple() {
    // Delegate before vote cast

    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user_delegatee = TestUser::new();
    let user_delegator = TestUser::new();

    let channel = TestChannel::new(&mut bench, &user_delegatee, Keypair::new()).await;
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

    let mut governance =
        TestGovernance::new(&mut bench, &channel.channel, &channel.authority).await;
    governance.with_native_treasury(&mut bench).await;

    let (proposal, rule, _destination) = TestProposal::new_transfer_proposal(
        &mut bench,
        &user_delegatee,
        &channel,
        &governance,
        &governance_token,
        1,
    )
    .await;

    // Enable the delegatee
    user_delegatee
        .create_delegatee(&mut bench, &governance_token, &rule)
        .await;

    // Delegate so that the delegatee now controls 51% of supply
    user_delegator
        .delegate(&mut bench, &user_delegatee, &governance_token, &rule, &50)
        .await;

    // vote for the transaction option
    proposal
        .vote(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &governance_token,
            &rule,
        )
        .await;

    proposal
        .vote_with_delegate(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &governance_token,
            &rule,
        )
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    ); // Enough votes now since delegated

    user_delegator
        .undelegate_history(
            &mut bench,
            &user_delegatee,
            &proposal,
            &governance_token,
            &rule,
        )
        .await;

    user_delegator
        .undelegate(&mut bench, &user_delegatee, &governance_token, &rule, &49)
        .await;
}

#[tokio::test]
async fn success_delegate_synchronization() {
    // Delegate when vote already cast.
    // Call synchronization funcionality to update previously casted vote
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user_delegatee = TestUser::new();
    let user_delegator = TestUser::new();

    let channel = TestChannel::new(&mut bench, &user_delegatee, Keypair::new()).await;
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
        .deposit_governance_tokens(&mut bench, 1, &governance_token)
        .await;

    let mut governance =
        TestGovernance::new(&mut bench, &channel.channel, &channel.authority).await;
    governance.with_native_treasury(&mut bench).await;

    let (proposal, rule, _destination) = TestProposal::new_transfer_proposal(
        &mut bench,
        &user_delegatee,
        &channel,
        &governance,
        &governance_token,
        1,
    )
    .await;

    // Enable the delegatee
    user_delegatee
        .create_delegatee(&mut bench, &governance_token, &rule)
        .await;

    // vote for the transaction option
    proposal
        .vote(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &governance_token,
            &rule,
        )
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(proposal.get_state(&mut bench).await, ProposalState::Voting); // Not enought votes

    // Delegate so that the delegatee now controls 51% of supply
    user_delegator
        .delegate(&mut bench, &user_delegatee, &governance_token, &rule, &50)
        .await;

    // vote for the transaction option
    proposal
        .vote(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &governance_token,
            &rule,
        )
        .await;

    proposal
        .vote_with_delegate(
            &mut bench,
            &vec![1],
            &user_delegatee,
            &governance_token,
            &rule,
        )
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    ); // Enough votes now since delegated
}
