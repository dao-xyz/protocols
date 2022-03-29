use crate::{
    social::utils::{TestChannel, TestPost, TestUser},
    utils::program_test,
};
use lgovernance::{
    find_create_rule_associated_program_address,
    instruction::{create_post_transaction, CreatePostType},
    state::post::{Action, ActionStatus, CreateRule, VotingRuleUpdate},
    state::rules::{deserialize_action_rule_account, AcceptenceCriteria, ActionType},
    Vote,
};
use solana_program::{instruction::InstructionError, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    signer::Signer,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use std::time::Duration;

use super::utils::{assert_action_status, execute_post, time_since_epoch};

#[tokio::test]
async fn create_event() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let total_supply = 100;
    let test_user = TestUser::new(&mut banks_client, &payer, &recent_blockhash).await;
    let test_channel =
        TestChannel::new(&test_user, &mut banks_client, &payer, &recent_blockhash).await;
    test_channel
        .mint_to(
            total_supply,
            &payer.pubkey(),
            &mut banks_client,
            &payer,
            &recent_blockhash,
        )
        .await;
    test_channel
        .mint_to(
            total_supply,
            &payer.pubkey(),
            &mut banks_client,
            &payer,
            &recent_blockhash,
        )
        .await;

    let expires_in_sec = 1;
    let expires_at = time_since_epoch() + expires_in_sec;
    let custom_rule_key = Pubkey::new_unique();
    let (_custom_rule_address, _) = find_create_rule_associated_program_address(
        &lpost::id(),
        &ActionType::CustomEvent(custom_rule_key),
        &test_channel.channel,
    );
    let action_type = ActionType::CustomEvent(custom_rule_key);

    // Create the rule for the event
    let create_rule_post = TestPost::new(&test_channel.channel).await;
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &create_rule_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageRule(VotingRuleUpdate::create(
                    CreateRule {
                        channel: test_channel.channel,
                        name: Some("Custom event".into()),
                        action: action_type.clone(),
                        criteria: AcceptenceCriteria::default(),
                        info: Some("info".into()),
                    },
                    &test_channel.channel,
                    &lpost::id(),
                )),
            },
            &create_rule_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    create_rule_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &create_rule_post.post,
        &ActionStatus::Pending,
    )
    .await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &test_channel,
        &create_rule_post,
    )
    .await
    .unwrap();

    // assert post is approved
    assert_action_status(
        &mut banks_client,
        &create_rule_post.post,
        &ActionStatus::Approved,
    )
    .await;

    // unvote to get tokens bak
    create_rule_post
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    deserialize_action_rule_account(
        &*banks_client
            .get_account(
                find_create_rule_associated_program_address(
                    &lpost::id(),
                    &action_type,
                    &test_channel.channel,
                )
                .0,
            )
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap();

    let expires_at = time_since_epoch() + expires_in_sec;

    // Create post event with invalid action
    let create_post_with_invalid_action = TestPost::new(&test_channel.channel).await;
    let mut transaction_post_invalid = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &create_post_with_invalid_action.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::CustomEvent {
                    data: vec![1, 2, 3],
                    event_type: Pubkey::new_unique(),
                },
            },
            &create_post_with_invalid_action.source,
        )],
        Some(&payer.pubkey()),
    );

    transaction_post_invalid.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post_invalid)
        .await
        .unwrap();

    create_post_with_invalid_action
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;

    let err = execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &test_channel,
        &create_post_with_invalid_action,
    )
    .await
    .unwrap_err();
    match err {
        TransportError::TransactionError(transaction_error) => match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::InvalidArgument => {}
                e => panic!("Wrong error type: {}", e),
            },
            _ => panic!("Wrong error type"),
        },
        _ => panic!("Wrong error type"),
    };

    // unvote to get tokens bak
    create_post_with_invalid_action
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    // Create post with valid action
    let create_post_valid_action = TestPost::new(&test_channel.channel).await;
    let mut transaction_post_valid = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &create_post_valid_action.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::CustomEvent {
                    data: vec![1, 2, 3],
                    event_type: custom_rule_key,
                },
            },
            &create_post_valid_action.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post_valid.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post_valid)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &create_post_valid_action.post,
        &ActionStatus::Pending,
    )
    .await;

    create_post_valid_action
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &test_channel,
        &create_post_valid_action,
    )
    .await
    .unwrap();

    // assets post is approved
    assert_action_status(
        &mut banks_client,
        &create_post_valid_action.post,
        &ActionStatus::Approved,
    )
    .await;
}
