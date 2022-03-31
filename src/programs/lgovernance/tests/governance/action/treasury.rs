use crate::{
    social::utils::{TestChannel, TestPost, TestUser},
    utils::program_test,
};
use lgovernance::{
    find_create_scope_associated_program_address,
    instruction::{create_post_transaction, CreatePostType},
    state::post::{Action, ActionStatus, Createscope, TreasuryAction, VotingScopeUpdate},
    state::scopes::{
        deserialize_action_scope_account, AcceptenceCriteria, ActionType, TreasuryActionType,
    },
    Vote,
};

use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use std::time::Duration;

use super::utils::{assert_action_status, execute_post, time_since_epoch};

#[tokio::test]
async fn success_create_and_transfer() {
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

    let expires_in_sec = 1;
    let expires_at = time_since_epoch() + expires_in_sec;

    let treasury_create_action_type = ActionType::Treasury(TreasuryActionType::Create);

    // Create the scope for the event
    let create_event_scope_post = TestPost::new(&test_channel.channel).await;
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &create_event_scope_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageScope(VotingScopeUpdate::create(
                    CreateScope {
                        channel: test_channel.channel,
                        name: None,
                        action: treasury_create_action_type.clone(),
                        criteria: AcceptenceCriteria::default(),
                        info: None,
                    },
                    &test_channel.channel,
                    &lpost::id(),
                )),
            },
            &create_event_scope_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    create_event_scope_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &create_event_scope_post.post,
        &ActionStatus::Pending,
    )
    .await;
    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &test_channel,
        &create_event_scope_post,
    )
    .await
    .unwrap();

    // assert post is approved
    assert_action_status(
        &mut banks_client,
        &create_event_scope_post.post,
        &ActionStatus::Approved,
    )
    .await;

    deserialize_action_scope_account(
        &*banks_client
            .get_account(
                find_create_scope_associated_program_address(
                    &lpost::id(),
                    &treasury_create_action_type,
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

    // Unvote to re require tokens
    create_event_scope_post
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply)
        .await;

    let expires_at = time_since_epoch() + expires_in_sec;
    let governence_mint = test_channel
        .get_channel_account(&mut banks_client)
        .await
        .governence_mint;
    // Create post action to create a treasury to hold stale/not used governence tokens
    let create_treasury_post = TestPost::new(&test_channel.channel).await;
    let mut transaction_create_treasury = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
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
        &test_channel,
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
        test_channel
            .get_treasury_account(&mut banks_client, &test_channel.mint)
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
            &test_channel.get_treasury_address(&governence_mint),
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
        test_channel
            .get_treasury_account(&mut banks_client, &test_channel.mint)
            .await
            .amount,
        1
    );

    // Create a proposal that allows treasury transfers
    let treasury_transfer_scope_post = TestPost::new(&test_channel.channel).await;
    let mut transaction_transfer_treasury_scope_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &treasury_transfer_scope_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::ManageScope(VotingScopeUpdate::create(
                    CreateScope {
                        action: ActionType::Treasury(TreasuryActionType::Transfer {
                            from: Some(test_channel.get_treasury_address(&test_channel.mint)),
                            to: Some(get_associated_token_address(
                                &payer.pubkey(),
                                &test_channel.mint,
                            )),
                        }),
                        channel: test_channel.channel,
                        criteria: AcceptenceCriteria::default(),
                        info: None,
                        name: None,
                    },
                    &test_channel.channel,
                    &lpost::id(),
                )),
            },
            &treasury_transfer_scope_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_transfer_treasury_scope_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_transfer_treasury_scope_post)
        .await
        .unwrap();

    treasury_transfer_scope_post
        .vote(&mut banks_client, &payer, Vote::Up, total_supply - 1)
        .await;

    tokio::time::sleep(Duration::from_millis(expires_in_sec + 10)).await;
    assert_action_status(
        &mut banks_client,
        &treasury_transfer_scope_post.post,
        &ActionStatus::Pending,
    )
    .await;

    execute_post(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &test_channel,
        &treasury_transfer_scope_post,
    )
    .await
    .unwrap();

    // assets post is approved
    assert_action_status(
        &mut banks_client,
        &treasury_transfer_scope_post.post,
        &ActionStatus::Approved,
    )
    .await;

    // unvote to reclaim tokens
    treasury_transfer_scope_post
        .unvote(&mut banks_client, &payer, Vote::Up, total_supply - 1)
        .await;

    // check that the scope exist

    let transfer_treasury_scope_address = find_create_scope_associated_program_address(
        &lpost::id(),
        &ActionType::Treasury(TreasuryActionType::Transfer {
            from: Some(test_channel.get_treasury_address(&test_channel.mint)),
            to: Some(get_associated_token_address(
                &payer.pubkey(),
                &test_channel.mint,
            )),
        }),
        &test_channel.channel,
    )
    .0;

    let _scope = deserialize_action_scope_account(
        &*banks_client
            .get_account(transfer_treasury_scope_address)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap();

    // Now lets create a proposal that initiates transfer from treasury back to payer
    // Create post action to transfer treasury governence token back to payer
    let create_treasury_transfer_post = TestPost::new(&test_channel.channel).await;
    let mut transaction_transfer_treasury = Transaction::new_with_payer(
        &[create_post_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &test_user.user,
            &test_channel.channel,
            &test_channel.mint,
            &create_treasury_transfer_post.hash,
            &CreatePostType::ActionPost {
                expires_at,
                action: Action::Treasury(TreasuryAction::transfer(
                    &test_channel.get_treasury_address(&test_channel.mint),
                    &get_associated_token_address(&payer.pubkey(), &test_channel.mint),
                    1,
                    &lpost::id(),
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
        &test_channel,
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

    // assert treasury is empty again
    assert_eq!(
        test_channel
            .get_treasury_account(&mut banks_client, &test_channel.mint)
            .await
            .amount,
        0
    );

    assert_eq!(
        test_user
            .get_token_account(&mut banks_client, &test_channel.mint)
            .await
            .amount,
        1
    );
}
