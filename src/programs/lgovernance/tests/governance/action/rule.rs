use crate::{governance::utils::TestToken, utils::program_test};
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};

use std::time::{SystemTime, UNIX_EPOCH};

fn time_since_epoch() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs()
}

#[tokio::test]
async fn approved_create_rule() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let total_supply = 100;
    // let governance_token = TestToken::new(&mut banks_client, &payer, &recent_blockhash).await;
    /*  let test_user = TestUser::new(&mut banks_client, &payer, &recent_blockhash).await;
    let test_channel = TestChannel::new(
        &test_user,
        &governance_token.mint,
        &mut banks_client,
        &payer,
        &recent_blockhash,
    )
    .await;
    let test_post = TestPost::new(&test_channel.channel).await;
    governance_token
        .mint_to(
            total_supply,
            &test_user.payer,
            &mut banks_client,
            &payer,
            &recent_blockhash,
        )
        .await;

    test_user
        .deposit_governance_tokens(
            &mut banks_client,
            &payer,
            &recent_blockhash,
            total_supply,
            &governance_token.mint,
        )
        .await;

    let _expires_in_sec = 1;
    let rule_id = Pubkey::new_unique();
    let (_custom_rule_address, _) =
        find_create_rule_associated_program_address(&lpost::id(), &rule_id);
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_proposal(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_post.hash,
            VoteType::SingleChoice,
            vec![],
            &test_user.get_token_owner_record_address(&mut banks_client, &governance_token.mint),
            &test_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();
         */
    /* expires_at,
    action: Action::ManageRule(VotingRuleUpdate::create(
        CreateRule {
            channel: test_channel.channel,
            name: Some("Some rule".into()),
            criteria: AcceptenceCriteria::default(),
            info: Some("info".into()),
            id: rule_id,
            instruction_condition: InstructionConditional {},
            instruction_program_id: lpost::id(),
            vote_mint: governance_token.mint,
        },
        &test_channel.channel,
        &lpost::id(),
    )), */

    /*  test_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(&mut banks_client, &test_post.post, &ActionStatus::Pending).await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &test_channel,
        &test_post,
    )
    .await
    .unwrap();

    // assets post is approved
    assert_action_status(&mut banks_client, &test_post.post, &ActionStatus::Approved).await;

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
    .unwrap(); */
}

/*
#[tokio::test]
async fn rejected_create_rule() {
    let program = program_test();
    let total_supply = 100;
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let test_user = TestUser::new(&mut banks_client, &payer, &recent_blockhash).await;
    let test_channel =
        TestChannel::new(&test_user, &mut banks_client, &payer, &recent_blockhash).await;
    let test_post = TestPost::new(&test_channel.channel).await;
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

    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &test_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageRule(VotingRuleUpdate::create(
                    CreateRule {
                        channel: test_channel.channel,
                        name: Some("Custom event".into()),
                        action: ActionType::CustomEvent(custom_rule_key),
                        criteria: AcceptenceCriteria::default(),
                        info: Some("info".into()),
                    },
                    &test_channel.channel,
                    &lpost::id(),
                )),
            },
            &test_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(&mut banks_client, &test_post.post, &ActionStatus::Pending).await;

    let mut execute_post = Transaction::new_with_payer(
        &[create_post_execution_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_post.post,
            &test_post.get_post_account(&mut banks_client).await,
            &test_channel.mint,
        )],
        Some(&payer.pubkey()),
    );
    execute_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(execute_post)
        .await
        .unwrap();

    // assets post is rejected since no voting
    assert_action_status(&mut banks_client, &test_post.post, &ActionStatus::Rejected).await;
} */
