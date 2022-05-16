use lsocial::state::{
    channel::ChannelType,
    channel_authority::{AuthorityCondition, AuthorityType},
    post::PostContent,
    vote_record::Vote,
};

use solana_program_test::*;

use crate::bench::ProgramTestBench;
use crate::utils::program_test;

use super::utils::{TestAuthority, TestChannel, TestPost, TestTagRecordFactory, TestUser};
/*
#[tokio::test]
async fn success_update_authority() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    let channel_name = "Channel";

    let (channel_account_address, authority) = create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        channel_name,
        &payer,
        &None,
        &ActivityAuthority::AuthorityByTag {
            tag: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
        },
        Some("link".into()),
    )
    .await
    .unwrap();

    let new_link =
        "ipfs://kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_update_info_transacation(
                &lsocial::id(),
                channel_name,
                Some(new_link.into()),
                &authority.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &authority],
            recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify channel changed

    let channel_account_info = banks_client
        .get_account(channel_account_address)
        .await
        .expect("get_channel")
        .expect("user not found");
    let user = deserialize_channel_account(&channel_account_info.data).unwrap();
    assert_eq!(user.link, Some(new_link.into()));
}

#[tokio::test]
async fn fail_already_exist() {
    let mut program = program_test();
    let wrong_payer = Keypair::new();
    program.add_account(
        wrong_payer.pubkey(),
        Account {
            lamports: 1000000,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let channel_name = "Channel";

    create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        channel_name,
        &payer,
        &None,
        &ActivityAuthority::AuthorityByTag {
            tag: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
        },
        Some("link".into()),
    )
    .await
    .unwrap();
    let latest_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    // Same transaction again
    let err = create_and_verify_channel(
        &mut banks_client,
        &payer,
        &latest_blockhash,
        channel_name,
        &payer,
        &None,
        &ActivityAuthority::AuthorityByTag {
            tag: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
        },
        Some("link".into()),
    )
    .await
    .unwrap_err();

    match err {
        TransportError::TransactionError(transaction_error) => match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::InvalidAccountData => {}
                _ => panic!("Wrong error type"),
            },
            _ => panic!("Wrong error type"),
        },
        _ => panic!("Wrong error type"),
    };
}

#[tokio::test]
async fn fail_update_info_wrong_authority() {
    let mut program = program_test();
    let wrong_authority = Keypair::new();
    program.add_account(
        wrong_authority.pubkey(),
        Account {
            lamports: 1000000,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let channel_name = "Channel";

    create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        channel_name,
        &payer,
        &None,
        &ActivityAuthority::AuthorityByTag {
            tag: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
        },
        Some("link".into()),
    )
    .await
    .unwrap();

    let new_link =
        "ipfs://kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";

    let err = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_update_info_transacation(
                &lsocial::id(),
                channel_name,
                Some(new_link.into()),
                &wrong_authority.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &wrong_authority],
            recent_blockhash,
        ))
        .await
        .unwrap_err();
    match err {
        TransportError::TransactionError(transaction_error) => match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::InvalidAccountData => {}
                _ => panic!("Wrong error type"),
            },
            _ => panic!("Wrong error type"),
        },
        _ => panic!("Wrong error type"),
    };
}

#[tokio::test]
async fn fail_update_authority_wrong_authority() {
    let mut program = program_test();
    let wrong_authority = Keypair::new();
    program.add_account(
        wrong_authority.pubkey(),
        Account {
            lamports: 1000000,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let channel_name = "Channel";

    create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        channel_name,
        &payer,
        &None,
        &ActivityAuthority::AuthorityByTag {
            tag: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
        },
        Some("link".into()),
    )
    .await
    .unwrap();

    let err = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_update_authority_transacation(
                &lsocial::id(),
                channel_name,
                &Pubkey::new_unique(),
                &wrong_authority.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &wrong_authority],
            recent_blockhash,
        ))
        .await
        .unwrap_err();
    match err {
        TransportError::TransactionError(transaction_error) => match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::InvalidAccountData => {}
                _ => panic!("Wrong error type"),
            },
            _ => panic!("Wrong error type"),
        },
        _ => panic!("Wrong error type"),
    };
} */

/*
#[tokio::test]
async fn fail_update_not_signer() {
    let mut program = program_test();
    let wrong_payer = Keypair::new();
    program.add_account(
        wrong_payer.pubkey(),
        Account {
            lamports: 1000000,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let username = "name";
    let user = create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        username,
        "profile",
    )
    .await;
    let channel_name = "Channel";
    create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        channel_name,
        &user,
        Some("link".into()),
    )
    .await;
    let (channel, _) = get_channel_program_address(&lsocial::id(), channel_name).unwrap();

    let err = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: lsocial::id(),
                data: SocialInstruction::ChannelInstruction(ChannelInstruction::UpdateChannel {
                    link: Some("new link".into()),
                })
                .try_to_vec()
                .unwrap(),
                accounts: vec![
                    AccountMeta::new(wrong_payer.pubkey(), false),
                    AccountMeta::new(user, false),
                    AccountMeta::new(channel, false),
                ],
            }],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        ))
        .await
        .unwrap_err();
    match err {
        TransportError::TransactionError(transaction_error) => match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::MissingRequiredSignature => {}
                _ => panic!("Wrong error type"),
            },
            _ => panic!("Wrong error type"),
        },
        _ => panic!("Wrong error type"),
    };
}
 */
// TODO add tests for "bad" channel names: padding, already exist etc

#[tokio::test]
pub async fn success_authority_by_tag() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();

    let tag_record_factory = TestTagRecordFactory::new(&mut bench).await;

    // create a collection
    let (test_channel, collection_authority) = TestChannel::new(
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

    // create a channel
    let (test_channel, authority) = TestChannel::new(
        &mut bench,
        &user,
        None,
        &ChannelType::PostStream,
        Some(&test_channel),
        Some(&create_channel_authority),
    )
    .await;

    // Add a new authority
    let admin_signer = authority
        .get_signing_authority(&mut bench, &signed_owner)
        .await;

    let new_authourity = TestAuthority::new(
        &mut bench,
        &test_channel,
        &vec![
            AuthorityType::CreatePost,
            AuthorityType::Comment,
            AuthorityType::Vote,
        ],
        &AuthorityCondition::Tag {
            record_factory: tag_record_factory.factory,
        },
        &admin_signer,
    )
    .await;

    let new_authorized_user = TestUser::new();
    tag_record_factory
        .new_record(&mut bench, &new_authorized_user)
        .await;

    // Create a post, vote and comment with the new authority
    let new_authorized_user_signer = (&new_authorized_user).into();

    let post_comment_vote_signer = new_authourity
        .get_signing_authority(&mut bench, &new_authorized_user_signer)
        .await;

    let post = TestPost::new(
        &mut bench,
        &test_channel,
        &new_authorized_user_signer,
        &PostContent::String("a".into()),
        None,
        &post_comment_vote_signer,
    )
    .await;

    // Vote
    post.vote(
        &mut bench,
        Vote::Up,
        &new_authorized_user_signer,
        &post_comment_vote_signer,
    )
    .await
    .unwrap();

    // Unvote
    post.unvote(
        &mut bench,
        &new_authorized_user_signer,
        &post_comment_vote_signer,
    )
    .await
    .unwrap();

    new_authourity.delete(&mut bench, &admin_signer).await;
}

// TODO add negative tests
