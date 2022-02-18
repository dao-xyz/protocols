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
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use super::utils::{assert_action_status, create_governence_token_and_supply_payer, execute_post};

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
                action: Action::ManageRule(VotingRuleUpdate::Create {
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
    assert_action_status(&mut banks_client, &socials.post, &ActionStatus::Pending).await;

    let mut execute_post = Transaction::new_with_payer(
        &[create_post_execution_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.post,
            &socials.get_post_account(&mut banks_client).await,
            &socials.governence_mint,
        )],
        Some(&payer.pubkey()),
    );
    execute_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(execute_post)
        .await
        .unwrap();

    // assets post is rejected since no voting
    assert_action_status(&mut banks_client, &socials.post, &ActionStatus::Rejected).await;
}

#[tokio::test]
async fn approved_create_rule() {
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
    let (_custom_rule_address, _) = find_create_rule_associated_program_address(
        &s2g::id(),
        &ActionType::CustomEvent(custom_rule_key),
        &socials.channel,
    );
    let action_type = ActionType::CustomEvent(custom_rule_key);
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

    // assets post is approved
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
}
