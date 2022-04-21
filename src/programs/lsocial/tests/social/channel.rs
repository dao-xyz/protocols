use lsocial::state::channel::ChannelType;
use shared::content::ContentSource;
use solana_program_test::*;

use crate::bench::ProgramTestBench;
use crate::utils::program_test;

use super::utils::{TestChannel, TestUser};

#[tokio::test]
pub async fn success() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    // create a channel
    let (test_collection, collection_authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Collection,
        None,
        None,
    )
    .await;

    // create a subchannel
    let create_channel_authority = collection_authority
        .get_signing_authority(&mut bench, &user)
        .await;
    TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Forum,
        Some(&test_collection),
        Some(&create_channel_authority),
    )
    .await;
}

#[tokio::test]
async fn update_info() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::Collection,
        None,
        None,
    )
    .await;
    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;

    test_channel
        .update_info(
            &mut bench,
            Some(ContentSource::String("hello".into())),
            &signing_authority,
        )
        .await;
}
