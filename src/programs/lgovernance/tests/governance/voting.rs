use crate::utils::program_test;
use lgovernance::{
    instruction::CreateProposalOptionType,
    state::{
        enums::ProposalState,
        native_treasury::get_native_treasury_address,
        proposal::{
            proposal_option::ProposalOption, proposal_transaction::ConditionedInstruction, VoteType,
        },
        scopes::scope::{ScopeCondition, ScopeConfig},
    },
};
use solana_program::{borsh::try_from_slice_unchecked, system_instruction, system_program};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

use super::super::bench::ProgramTestBench;
use super::utils::{TestChannel, TestGovernance, TestProposal, TestToken, TestUser};

#[tokio::test]
async fn success_vote_execute() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user = TestUser::new();

    let channel = TestChannel::new(&mut bench, &user, Keypair::new()).await;

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

    let mut governance =
        TestGovernance::new(&mut bench, &channel.channel, &channel.authority).await;
    governance.with_native_treasury(&mut bench).await;

    let transfer_amount = 1;
    let (proposal, scope, recipent_wallet) = TestProposal::new_transfer_proposal(
        &mut bench,
        &user,
        &channel,
        &governance,
        &governance_token,
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
        .vote(&mut bench, &vec![1], &user, &governance_token, &scope)
        .await;

    proposal.count_votes(&mut bench).await;

    assert_eq!(
        proposal.get_state(&mut bench).await,
        ProposalState::Succeeded
    );

    proposal
        .advance_clock_past_max_hold_up_time(&mut bench, 1)
        .await;

    proposal
        .execute_transactions(&mut bench, &channel.channel, 1)
        .await;

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
async fn success_vote_unvote() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;

    let user = TestUser::new();
    let channel = TestChannel::new(&mut bench, &user, Keypair::new()).await;
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

    let mut governance =
        TestGovernance::new(&mut bench, &channel.channel, &channel.authority).await;
    governance.with_native_treasury(&mut bench).await;

    let (proposal, scope, _recipent_wallet) = TestProposal::new_transfer_proposal(
        &mut bench,
        &user,
        &channel,
        &governance,
        &governance_token,
        1,
    )
    .await;

    // vote for the transaction option
    proposal
        .vote(&mut bench, &vec![0], &user, &governance_token, &scope)
        .await;

    let beneficiary = bench.with_wallet().await;
    let beneficiary_balance = bench
        .get_account(&beneficiary.address)
        .await
        .unwrap()
        .lamports;

    proposal
        .unvote(
            &mut bench,
            vec![0],
            &user,
            &governance_token,
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
