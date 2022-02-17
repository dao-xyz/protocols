use crate::{social::post::utils::SocialAccounts, utils::program_test};
use s2g::socials::post::{
    find_create_rule_associated_program_address, find_escrow_program_address,
    find_post_program_address,
    instruction::{create_post_execution_transaction, create_post_transaction, CreatePostType},
    state::{
        deserialize_post_account, AcceptenceCriteria, Action, ActionStatus, ActionType, CreateRule,
        PostType, VotingRuleUpdate,
    },
    Vote,
};
use solana_program::{pubkey::Pubkey, rent::Rent};
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn time_since_epoch() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    return since_the_epoch.as_secs();
}

#[tokio::test]
async fn rejected_create_rule() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey());
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;

    let expires_in_sec = 1;
    let expires_at = time_since_epoch() + expires_in_sec;
    let custom_rule_key = Pubkey::new_unique();
    let (_custom_rule_address, _) = find_create_rule_associated_program_address(
        &s2g::id(),
        &ActionType::CustomEvent(custom_rule_key),
        &socials.channel,
    );
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
                action: Action::ManageRule(VotingRuleUpdate::CreateRule {
                    rule: CreateRule {
                        channel: socials.channel,
                        name: Some("Custom event".into()),
                        action: ActionType::CustomEvent(custom_rule_key),
                        criteria: AcceptenceCriteria::default(),
                        info: Some("info".into()),
                    },
                    bump_seed: 123,
                }),
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

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    socials
        .assert_action_status(&mut banks_client, ActionStatus::Pending)
        .await;

    let mut execute_post = Transaction::new_with_payer(
        &[create_post_execution_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.post,
            &socials.get_post_account(&mut banks_client).await,
            &ActionType::CustomEvent(custom_rule_key),
            &socials.governence_mint,
        )],
        Some(&payer.pubkey()),
    );
    execute_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(execute_post)
        .await
        .unwrap();

    // assets post is approved
    socials
        .assert_action_status(&mut banks_client, ActionStatus::Approved)
        .await;

    // assert rule exist
}

#[tokio::test]
async fn approved_create_rule() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey());
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;

    let expires_in_sec = 1;
    let expires_at = time_since_epoch() + expires_in_sec;
    let custom_rule_key = Pubkey::new_unique();
    let (_custom_rule_address, _) = find_create_rule_associated_program_address(
        &s2g::id(),
        &ActionType::CustomEvent(custom_rule_key),
        &socials.channel,
    );
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
                action: Action::ManageRule(VotingRuleUpdate::CreateRule {
                    rule: CreateRule {
                        channel: socials.channel,
                        name: Some("Custom event".into()),
                        action: ActionType::CustomEvent(custom_rule_key),
                        criteria: AcceptenceCriteria::default(),
                        info: Some("info".into()),
                    },
                    bump_seed: 123,
                }),
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

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    socials
        .assert_action_status(&mut banks_client, ActionStatus::Pending)
        .await;

    let mut execute_post = Transaction::new_with_payer(
        &[create_post_execution_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.post,
            &socials.get_post_account(&mut banks_client).await,
            &ActionType::CustomEvent(custom_rule_key),
            &socials.governence_mint,
        )],
        Some(&payer.pubkey()),
    );
    execute_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(execute_post)
        .await
        .unwrap();

    // assets post is approved
    socials
        .assert_action_status(&mut banks_client, ActionStatus::Approved)
        .await;

    // assert rule exist
}
