use crate::{social::post::utils::SocialAccounts, utils::program_test};
use s2g::socials::post::{
    find_create_rule_associated_program_address,
    instruction::{create_post_execution_transaction, create_post_transaction, CreatePostType},
    state::{
        deserialize_action_rule_account, AcceptenceCriteria, Action, ActionStatus, ActionType,
        CreateRule, VotingRuleUpdate,
    },
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
use std::time::{SystemTime, UNIX_EPOCH};

use super::utils::{
    assert_action_status, create_governence_token_and_supply_payer, execute_post, time_since_epoch,
};

#[tokio::test]
async fn success() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let total_supply = 100;
    let mint = create_governence_token_and_supply_payer(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        total_supply,
    )
    .await;
    let socials = SocialAccounts::new_with_org(&payer.pubkey(), &mint.pubkey());
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, 0)
        .await;

    let expires_in_sec = 1;
    let expires_at = time_since_epoch() + expires_in_sec;
    let custom_rule_key = Pubkey::new_unique();

    let action_type = ActionType::TransferTreasury;

    // Create the rule for the event
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &socials.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageRule(VotingRuleUpdate::create(
                    &s2g::id(),
                    CreateRule {
                        channel: socials.channel,
                        name: Some("Custom event".into()),
                        action: action_type.clone(),
                        criteria: AcceptenceCriteria::default(),
                        info: Some("info".into()),
                    },
                    &socials.channel,
                )),
            },
            &socials.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    socials
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(&mut banks_client, &socials.post, &ActionStatus::Pending).await;

    execute_post(&mut banks_client, &payer, &recent_blockhash, &socials).await;

    // assert post is approved
    assert_action_status(&mut banks_client, &socials.post, &ActionStatus::Approved).await;

    deserialize_action_rule_account(
        &*banks_client
            .get_account(
                find_create_rule_associated_program_address(
                    &s2g::id(),
                    &action_type,
                    &socials.channel,
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
    let mut transaction_post_invalid = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &socials.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::CustomEvent {
                    data: vec![1, 2, 3],
                    event_type: Pubkey::new_unique(),
                },
            },
            &socials.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post_invalid.sign(&[&payer], recent_blockhash);
    let err = banks_client
        .process_transaction(transaction_post_invalid)
        .await
        .unwrap_err();
    match err {
        TransportError::TransactionError(transaction_error) => match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::InvalidArgument => {}
                _ => panic!("Wrong error type"),
            },
            _ => panic!("Wrong error type"),
        },
        _ => panic!("Wrong error type"),
    };

    // Create post with valid action
    let mut transaction_post_valid = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &socials.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::CustomEvent {
                    data: vec![1, 2, 3],
                    event_type: custom_rule_key,
                },
            },
            &socials.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post_valid.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post_valid)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(&mut banks_client, &socials.post, &ActionStatus::Pending).await;

    execute_post(&mut banks_client, &payer, &recent_blockhash, &socials).await;

    // assets post is approved
    assert_action_status(&mut banks_client, &socials.post, &ActionStatus::Approved).await;
}
