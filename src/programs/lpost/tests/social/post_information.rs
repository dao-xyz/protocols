use super::{
    super::utils::program_test,
    utils::{create_tag, create_tag_record, TestPost},
};
use crate::social::utils::TestChannel;
use lchannel::state::ActivityAuthority;
use lpost::{
    error::PostError,
    state::{post::PostContent, vote_record::Vote},
};
use shared::content::ContentSource;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::TransactionError};

#[tokio::test]
async fn success_round_trip() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let authority = Keypair::new(); // Context for creating tag records
    let authority_tag = create_tag(&mut banks_client, &payer, &recent_blockhash, "tag").await;
    let test_channel = TestChannel::new(
        &ActivityAuthority::AuthorityByTag {
            tag: authority_tag,
            authority: authority.pubkey(),
        },
        &mut banks_client,
        &payer,
        &recent_blockhash,
    )
    .await;

    // Create a record so we have authority to vote and create posts
    create_tag_record(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &authority_tag,
        &payer,
        &authority,
    )
    .await;

    let test_post = TestPost::new(
        &test_channel,
        &PostContent::Info {
            content: ContentSource::String("Hello".into()),
        },
        &payer,
        &payer,
        &mut banks_client,
        &recent_blockhash,
    )
    .await;

    test_post
        .vote(
            Vote::Up,
            &payer,
            &mut banks_client,
            &payer,
            &recent_blockhash,
        )
        .await
        .unwrap();

    test_post.assert_votes(&mut banks_client, 1, 0).await;
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    // Test failing revotes
    assert_eq!(
        test_post
            .vote(
                Vote::Up,
                &payer,
                &mut banks_client,
                &payer,
                &recent_blockhash
            )
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PostError::VoteAlreadyExist as u32)
        )
    );
    assert_eq!(
        test_post
            .vote(
                Vote::Down,
                &payer,
                &mut banks_client,
                &payer,
                &recent_blockhash
            )
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PostError::VoteAlreadyExist as u32)
        )
    );

    // unvote
    test_post
        .unvote(&payer, &mut banks_client, &payer, &recent_blockhash)
        .await
        .unwrap();

    test_post.assert_votes(&mut banks_client, 0, 0).await;
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    // test revote again, but this time downvote
    test_post
        .vote(
            Vote::Down,
            &payer,
            &mut banks_client,
            &payer,
            &recent_blockhash,
        )
        .await
        .unwrap();
    test_post.assert_votes(&mut banks_client, 0, 1).await;
}
