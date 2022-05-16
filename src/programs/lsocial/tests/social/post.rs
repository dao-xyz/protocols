use super::utils::{TestPost, TestSignerMaybeForMe};
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
use solana_sdk::{signature::Keypair, signer::Signer};

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
    let signed_owner = (&user).into();
    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &signed_owner)
        .await;
    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::PostStream,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;

    let signing_authority = authority
        .get_signing_authority(&mut bench, &signed_owner)
        .await;
    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &signed_owner,
        &PostContent::String("a".into()),
        None,
        &signing_authority,
    )
    .await;

    let comment_1 = TestPost::new(
        &mut bench,
        &test_channel,
        &signed_owner,
        &PostContent::String("a".into()),
        Some(&post),
        &signing_authority,
    )
    .await;

    let _comment_2 = TestPost::new(
        &mut bench,
        &test_channel,
        &signed_owner,
        &PostContent::String("a".into()),
        Some(&post),
        &signing_authority,
    )
    .await;

    let _comment_1_1 = TestPost::new(
        &mut bench,
        &test_channel,
        &signed_owner,
        &PostContent::String("a".into()),
        Some(&comment_1),
        &signing_authority,
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

    let signed_owner = (&user).into();
    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &signed_owner)
        .await;

    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::PostStream,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;

    let signing_authority = authority
        .get_signing_authority(&mut bench, &signed_owner)
        .await;

    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &signed_owner,
        &PostContent::String("a".into()),
        None,
        &signing_authority,
    )
    .await;

    // Vote
    post.vote(&mut bench, Vote::Up, &signed_owner, &signing_authority)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Vote again fail
    assert_eq!(
        post.vote(&mut bench, Vote::Up, &signed_owner, &signing_authority)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteAlreadyExist as u32)
    );

    // Unvote
    post.unvote(&mut bench, &signed_owner, &signing_authority)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Unvote again fail
    assert_eq!(
        post.unvote(&mut bench, &signed_owner, &signing_authority)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteDoesNotExist as u32)
    );
}

#[tokio::test]
async fn success_vote_unvote_sign_for_me() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let admin = TestUser::new();
    let (test_collection, collection_authority) = TestChannel::new(
        &mut bench,
        &admin,
        None,
        &ChannelType::Collection,
        None,
        None,
    )
    .await;

    let signed_owner = (&admin).into();

    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &signed_owner)
        .await;

    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &admin,
        None,
        &ChannelType::PostStream,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;

    let sign_for_me_signer = Keypair::new();

    let scope = test_channel.channel;

    let sign_for_me =
        TestSignForMe::new(&mut bench, &admin.keypair, &sign_for_me_signer, &scope).await;

    let signer_as_sign_for_me = TestSignerMaybeForMe {
        original_signer: &admin.keypair,
        sign_for_me: Some(&sign_for_me),
    };

    let signing_authority = authority
        .get_signing_authority(&mut bench, &signer_as_sign_for_me)
        .await;

    println!("EXPECTED SIGNER {}", sign_for_me_signer.pubkey());
    println!("CHANNEL {}", test_channel.channel);
    let signing_owner = TestSignerMaybeForMe {
        original_signer: &admin.keypair,
        sign_for_me: Some(&sign_for_me),
    };

    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &signing_owner,
        &PostContent::String("a".into()),
        None,
        &signing_authority,
    )
    .await;

    // Vote
    post.vote(&mut bench, Vote::Up, &signing_owner, &signing_authority)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Vote again fail
    assert_eq!(
        post.vote(&mut bench, Vote::Up, &signing_owner, &signing_authority)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteAlreadyExist as u32)
    );

    // Unvote
    post.unvote(&mut bench, &signing_owner, &signing_authority)
        .await
        .unwrap();

    bench.advance_clock().await;

    // Unvote again fail
    assert_eq!(
        post.unvote(&mut bench, &signing_owner, &signing_authority)
            .await
            .unwrap_err(),
        ProgramError::Custom(SocialError::VoteDoesNotExist as u32)
    );
}
