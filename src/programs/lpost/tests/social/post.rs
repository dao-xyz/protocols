use super::utils::TestPost;
use crate::{
    bench::ProgramTestBench,
    social::utils::{TestChannel, TestUser},
    utils::program_test,
};

use lsocial::{
    error::SocialError,
    state::{post::PostContent, vote_record::Vote},
};
use shared::content::ContentSource;
use solana_program::program_error::ProgramError;
use solana_program_test::*;

#[tokio::test]
async fn success_post_comment_comment() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_channel, authority) = TestChannel::new(&mut bench, &user, None, None, None).await;

    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;
    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::ContentSource(ContentSource::String("a".into())),
        None,
        &signing_authority,
    )
    .await;

    let comment_1 = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::ContentSource(ContentSource::String("a".into())),
        Some(&post),
        &signing_authority,
    )
    .await;

    let _comment_2 = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::ContentSource(ContentSource::String("a".into())),
        Some(&post),
        &signing_authority,
    )
    .await;

    let _comment_1_1 = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::ContentSource(ContentSource::String("a".into())),
        Some(&comment_1),
        &signing_authority,
    )
    .await;
}

#[tokio::test]
async fn success_vote_unvote() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_channel, authority) = TestChannel::new(&mut bench, &user, None, None, None).await;
    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;

    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::ContentSource(ContentSource::String("a".into())),
        None,
        &signing_authority,
    )
    .await;

    // Vote
    post.vote(&mut bench, Vote::Up, &user, &signing_authority)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Vote again fail
    assert_eq!(
        post.vote(&mut bench, Vote::Up, &user, &signing_authority)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteAlreadyExist as u32)
    );

    // Unvote
    post.unvote(&mut bench, &user, &signing_authority)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Unvote again fail
    assert_eq!(
        post.unvote(&mut bench, &user, &signing_authority)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteDoesNotExist as u32)
    );
}
