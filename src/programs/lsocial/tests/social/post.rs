use super::utils::TestPost;
use crate::{
    bench::ProgramTestBench,
    social::utils::{TestChannel, TestSignForMe, TestUser},
    utils::program_test,
};

use borsh::BorshSerialize;
use lsocial::{
    error::SocialError,
    instruction::{CreateVoteConfig, PostInstruction},
    state::{channel::ChannelType, post::PostContent, vote_record::Vote},
};

use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use solana_program_test::*;
use solana_sdk::signature::Keypair;

#[tokio::test]
async fn success_xxx() {
    let hash = Pubkey::new_unique().to_bytes();
    let q = (PostInstruction::CreatePost {
        hash: hash,
        post_bump_seed: 1,
        is_child: true,
        content: PostContent::String("a".into()),
        vote_config: CreateVoteConfig::Simple,
    })
    .try_to_vec();
    let x = q.unwrap();
    println!("XXX {}", x.len());
}
#[tokio::test]
async fn success_post_comment_comment() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_collection, collection_authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Collection,
        None,
        None,
    )
    .await;

    // create forum subchannel
    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &user)
        .await;
    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Forum,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;

    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;
    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::String("a".into()),
        None,
        &signing_authority,
        None,
    )
    .await;

    let comment_1 = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::String("a".into()),
        Some(&post),
        &signing_authority,
        None,
    )
    .await;

    let _comment_2 = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::String("a".into()),
        Some(&post),
        &signing_authority,
        None,
    )
    .await;

    let _comment_1_1 = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::String("a".into()),
        Some(&comment_1),
        &signing_authority,
        None,
    )
    .await;
}

#[tokio::test]
async fn success_vote_unvote() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_collection, collection_authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Collection,
        None,
        None,
    )
    .await;

    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &user)
        .await;

    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Forum,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;

    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;

    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::String("a".into()),
        None,
        &signing_authority,
        None,
    )
    .await;

    // Vote
    post.vote(&mut bench, Vote::Up, &user, &signing_authority, None)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Vote again fail
    assert_eq!(
        post.vote(&mut bench, Vote::Up, &user, &signing_authority, None)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteAlreadyExist as u32)
    );

    // Unvote
    post.unvote(&mut bench, &user, &signing_authority, None)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Unvote again fail
    assert_eq!(
        post.unvote(&mut bench, &user, &signing_authority, None)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteDoesNotExist as u32)
    );
}

async fn success_vote_unvote_sign_for_me() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_collection, collection_authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Collection,
        None,
        None,
    )
    .await;

    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &user)
        .await;

    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Forum,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;

    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;
    let sign_for_me_signer = Keypair::new();
    let scope = test_channel.channel;

    let scope = test_channel.channel;
    let sign_for_me = TestSignForMe::new(
        &mut bench,
        signing_authority.signer,
        &sign_for_me_signer,
        &scope,
    )
    .await;

    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &user)
        .await;

    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &user,
        &PostContent::String("a".into()),
        None,
        &signing_authority,
        Some(&sign_for_me),
    )
    .await;

    // Vote
    post.vote(
        &mut bench,
        Vote::Up,
        &user,
        &signing_authority,
        Some(&sign_for_me),
    )
    .await
    .unwrap();

    bench.advance_clock().await;

    // Vote again fail
    assert_eq!(
        post.vote(&mut bench, Vote::Up, &user, &signing_authority, None)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteAlreadyExist as u32)
    );

    // Unvote
    post.unvote(&mut bench, &user, &signing_authority, Some(&sign_for_me))
        .await
        .unwrap();

    bench.advance_clock().await;

    // Unvote again fail
    assert_eq!(
        post.unvote(&mut bench, &user, &signing_authority, None)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteDoesNotExist as u32)
    );
}
