use super::utils::{assert_token_balance, SocialAccounts};
use crate::social::post::utils::TestPost;
use crate::social::post::utils::TestUser;
use crate::utils::program_test;
use s2g::socials::post::{
    find_escrow_program_address,
    instruction::{create_post_transaction, CreatePostType},
    Vote,
};
use solana_program::rent::Rent;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};

#[tokio::test]
async fn success_upvote() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey());
    let test_post = TestPost::new(&socials.channel);
    let test_user = TestUser::new(&payer.pubkey(), &test_post);

    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &test_post.hash,
            &CreatePostType::SimplePost,
            &test_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    let (escrow_account_info, _) = find_escrow_program_address(&s2g::id(), &test_post.post);
    let rent = Rent::default();
    let stake = 1000;

    // Stake some
    test_post
        .vote(&mut banks_client, &payer, Vote::Up, stake)
        .await;

    let escrow_account = banks_client
        .get_account(escrow_account_info)
        .await
        .unwrap()
        .unwrap();

    assert!(rent.is_exempt(escrow_account.lamports, escrow_account.data.len()));

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &test_user.upvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    test_post.assert_vote(&mut banks_client, stake, 0).await;

    // Stake more
    test_post
        .vote(&mut banks_client, &payer, Vote::Up, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake * 2,
    )
    .await;

    assert_token_balance(
        &mut banks_client,
        &test_user.upvote_token_account,
        stake * 2,
    )
    .await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake * 2).await;
    test_post.assert_vote(&mut banks_client, stake * 2, 0).await;

    // Unstake
    test_post
        .unvote(&mut banks_client, &payer, Vote::Up, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &test_user.upvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    test_post.assert_vote(&mut banks_client, stake, 0).await;

    // Unstake, same amount (we should now 0 token accounts)
    test_post
        .unvote(&mut banks_client, &payer, Vote::Up, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount,
    )
    .await;
    assert_token_balance(&mut banks_client, &test_user.upvote_token_account, 0).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, 0).await;
    test_post.assert_vote(&mut banks_client, 0, 0).await;
}

#[tokio::test]
async fn success_downvote() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey());
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;
    let test_post = TestPost::new(&socials.channel);
    let test_user = TestUser::new(&payer.pubkey(), &test_post);
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &test_post.hash,
            &CreatePostType::SimplePost,
            &test_post.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    let (escrow_account_info, _) = find_escrow_program_address(&s2g::id(), &test_post.post);
    let rent = Rent::default();
    let stake = 1000;

    // Stake some
    test_post
        .vote(&mut banks_client, &payer, Vote::Down, stake)
        .await;

    let escrow_account = banks_client
        .get_account(escrow_account_info)
        .await
        .unwrap()
        .unwrap();

    assert!(rent.is_exempt(escrow_account.lamports, escrow_account.data.len()));

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &test_user.downvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    test_post.assert_vote(&mut banks_client, 0, stake).await;

    // Stake more
    test_post
        .vote(&mut banks_client, &payer, Vote::Down, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake * 2,
    )
    .await;

    assert_token_balance(
        &mut banks_client,
        &test_user.downvote_token_account,
        stake * 2,
    )
    .await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake * 2).await;
    test_post.assert_vote(&mut banks_client, 0, stake * 2);

    // Unstake
    test_post
        .unvote(&mut banks_client, &payer, Vote::Down, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &test_user.downvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    test_post.assert_vote(&mut banks_client, 0, stake);

    // Unstake, same amount (we should now 0 token accounts)
    test_post
        .unvote(&mut banks_client, &payer, Vote::Down, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount,
    )
    .await;
    assert_token_balance(&mut banks_client, &test_user.downvote_token_account, 0).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, 0).await;
    test_post.assert_vote(&mut banks_client, 0, 0);
}
