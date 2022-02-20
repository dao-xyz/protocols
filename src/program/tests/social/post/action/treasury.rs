use crate::{
    social::post::utils::{SocialAccounts, TestPost, TestUser},
    utils::program_test,
};
use s2g::socials::post::{
    find_create_rule_associated_program_address, find_post_program_address,
    instruction::{create_post_execution_transaction, create_post_transaction, CreatePostType},
    state::{
        deserialize_action_rule_account, AcceptenceCriteria, Action, ActionStatus, ActionType,
        CreateRule, TreasuryAction, TreasuryActionType, VotingRuleUpdate,
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
use spl_associated_token_account::get_associated_token_address;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use super::utils::{
    assert_action_status, create_governence_token_and_supply_payer, execute_post, time_since_epoch,
};

#[tokio::test]
async fn success_create_and_transfer() {
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

    let treasury_create_action_type = ActionType::Treasury(TreasuryActionType::Create);

    // Create the rule for the event
    let create_event_rule_post = TestPost::new(&socials.channel);
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &create_event_rule_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageRule(VotingRuleUpdate::create(
                    CreateRule {
                        channel: socials.channel,
                        name: None,
                        action: treasury_create_action_type.clone(),
                        criteria: AcceptenceCriteria::default(),
                        info: None,
                    },
                    &socials.channel,
                    &s2g::id(),
                )),
            },
            &create_event_rule_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    create_event_rule_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &create_event_rule_post.post,
        &ActionStatus::Pending,
    )
    .await;
    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &socials,
        &create_event_rule_post,
    )
    .await
    .unwrap();

    // assert post is approved
    assert_action_status(
        &mut banks_client,
        &create_event_rule_post.post,
        &ActionStatus::Approved,
    )
    .await;

    deserialize_action_rule_account(
        &*banks_client
            .get_account(
                find_create_rule_associated_program_address(
                    &s2g::id(),
                    &treasury_create_action_type,
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

    // Unvote to re require tokens
    create_event_rule_post
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    let expires_at = time_since_epoch() + expires_in_sec;
    let governence_mint = socials
        .get_channel_account(&mut banks_client)
        .await
        .governence_mint;
    // Create post action to create a treasury to hold stale/not used governence tokens
    let create_treasury_post = TestPost::new(&socials.channel);
    let mut transaction_create_treasury = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &create_treasury_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::Treasury(TreasuryAction::Create {
                    mint: governence_mint,
                }),
            },
            &create_treasury_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_create_treasury.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_create_treasury)
        .await
        .unwrap();

    create_treasury_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &create_treasury_post.post,
        &ActionStatus::Pending,
    )
    .await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &socials,
        &create_treasury_post,
    )
    .await
    .unwrap();

    // assets post is approved
    assert_action_status(
        &mut banks_client,
        &create_treasury_post.post,
        &ActionStatus::Approved,
    )
    .await;

    // assert treasury exist
    assert_eq!(
        socials
            .get_treasury_account(&mut banks_client, &socials.governence_mint)
            .await
            .amount,
        0
    );

    // unvote to reclaim tokens
    create_treasury_post
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    // transfer some to the treasury
    let mut transaction_token_transfer = Transaction::new_with_payer(
        &[spl_token::instruction::transfer(
            &spl_token::id(),
            &get_associated_token_address(&payer.pubkey(), &governence_mint),
            &socials.get_treasury_address(&governence_mint),
            &payer.pubkey(),
            &[&payer.pubkey()],
            1,
        )
        .unwrap()],
        Some(&payer.pubkey()),
    );
    transaction_token_transfer.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_token_transfer)
        .await
        .unwrap();

    assert_eq!(
        socials
            .get_treasury_account(&mut banks_client, &socials.governence_mint)
            .await
            .amount,
        1
    );

    // Create a proposal that allows treasury transfers
    let treasury_transfer_rule_post = TestPost::new(&socials.channel);
    let mut transaction_transfer_treasury_rule_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &treasury_transfer_rule_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageRule(VotingRuleUpdate::create(
                    CreateRule {
                        action: ActionType::Treasury(TreasuryActionType::Transfer {
                            from: Some(socials.get_treasury_address(&socials.governence_mint)),
                            to: Some(get_associated_token_address(
                                &payer.pubkey(),
                                &socials.governence_mint,
                            )),
                        }),
                        channel: socials.channel,
                        criteria: AcceptenceCriteria::default(),
                        info: None,
                        name: None,
                    },
                    &socials.channel,
                    &s2g::id(),
                )),
            },
            &treasury_transfer_rule_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_transfer_treasury_rule_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_transfer_treasury_rule_post)
        .await
        .unwrap();

    treasury_transfer_rule_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply - 1)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &treasury_transfer_rule_post.post,
        &ActionStatus::Pending,
    )
    .await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &socials,
        &treasury_transfer_rule_post,
    )
    .await
    .unwrap();

    // assets post is approved
    assert_action_status(
        &mut banks_client,
        &treasury_transfer_rule_post.post,
        &ActionStatus::Approved,
    )
    .await;

    // unvote to reclaim tokens
    treasury_transfer_rule_post
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply - 1)
        .await;

    // check that the rule exist

    let transfer_treasury_rule_address = find_create_rule_associated_program_address(
        &s2g::id(),
        &ActionType::Treasury(TreasuryActionType::Transfer {
            from: Some(socials.get_treasury_address(&socials.governence_mint)),
            to: Some(get_associated_token_address(
                &payer.pubkey(),
                &socials.governence_mint,
            )),
        }),
        &socials.channel,
    )
    .0;

    let rule = deserialize_action_rule_account(
        &*banks_client
            .get_account(transfer_treasury_rule_address)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap();

    // Now lets create a proposal that initiates transfer from treasury back to payer
    // Create post action to transfer treasury governence token back to payer
    let create_treasury_transfer_post = TestPost::new(&socials.channel);
    let mut transaction_transfer_treasury = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &create_treasury_transfer_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::Treasury(TreasuryAction::transfer(
                    &socials.get_treasury_address(&socials.governence_mint),
                    &get_associated_token_address(&payer.pubkey(), &socials.governence_mint),
                    1,
                    &s2g::id(),
                )),
            },
            &create_treasury_transfer_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_transfer_treasury.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_transfer_treasury)
        .await
        .unwrap();

    create_treasury_transfer_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply - 1)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &create_treasury_transfer_post.post,
        &ActionStatus::Pending,
    )
    .await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &socials,
        &create_treasury_transfer_post,
    )
    .await
    .unwrap();

    // assets post is approved
    assert_action_status(
        &mut banks_client,
        &create_treasury_transfer_post.post,
        &ActionStatus::Approved,
    )
    .await;

    let test_user = TestUser::new(&payer.pubkey(), &create_treasury_transfer_post);

    // assert treasury is empty again
    assert_eq!(
        socials
            .get_treasury_account(&mut banks_client, &socials.governence_mint)
            .await
            .amount,
        0
    );

    assert_eq!(
        test_user
            .get_token_account(&mut banks_client, &socials.governence_mint)
            .await
            .amount,
        1
    );
}
