use shared::content::ContentSource;
use solana_program_test::*;

use crate::bench::ProgramTestBench;
use crate::utils::program_test;

use super::utils::{TestChannel, TestUser};
/*
use crate::utils::program_test;

pub fn deserialize_channel_account(data: &[u8]) -> std::io::Result<ChannelAccount> {
    let account: ChannelAccount = try_from_slice_unchecked(data)?;
    Ok(account)
}

pub async fn create_and_verify_channel(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    channel_name: &str,
    creator: &Keypair,
    parent_and_authority: &Option<(Pubkey, Pubkey)>,
    activity_authority: &ActivityAuthority,
    info: Option<ContentSource>,
) -> Result<(Pubkey, Keypair), TransportError> {
    let authority = Keypair::new();
    create_and_verify_channel_with_authority(
        banks_client,
        payer,
        recent_blockhash,
        channel_name,
        creator,
        parent_and_authority,
        activity_authority,
        info,
        authority,
    )
    .await
}
pub async fn create_and_verify_channel_with_authority(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    channel_name: &str,
    creator: &Keypair,
    parent_and_authority: &Option<(Pubkey, Pubkey)>,
    activity_authority: &ActivityAuthority,
    info: Option<ContentSource>,
    authority: Keypair,
) -> Result<(Pubkey, Keypair), TransportError> {
    let (channel_address_pda, _bump) =
        get_channel_program_address(&lsocial::id(), channel_name).unwrap();

    let mut transaction_create = Transaction::new_with_payer(
        &[create_channel(
            &lsocial::id(),
            channel_name,
            &creator.pubkey(),
            &authority.pubkey(),
            *parent_and_authority,
            activity_authority,
            info,
            &payer.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[payer, creator, &authority], *recent_blockhash);
    banks_client.process_transaction(transaction_create).await?;

    // Verify channel name
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account = deserialize_channel_account(&channel_account_info.data).unwrap();

    assert_eq!(channel_account.name.as_str(), channel_name);
    Ok((channel_address_pda, authority))
}
 */
#[tokio::test]
pub async fn success() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    // create a channel
    let (test_channel, authority) = TestChannel::new(&mut bench, &user, None, None, None).await;

    // create a subchannel
    let create_channel_authority = authority.get_signing_authority(&mut bench, &user).await;
    TestChannel::new(
        &mut bench,
        &user,
        None,
        Some(&test_channel),
        Some(&create_channel_authority),
    )
    .await;
}

#[tokio::test]
async fn success_update_info() {
    let mut bench = ProgramTestBench::start_new(program_test()).await;
    let user = TestUser::new();
    let (test_channel, authority) = TestChannel::new(&mut bench, &user, None, None, None).await;
    let signing_authority = authority.get_signing_authority(&mut bench, &user).await;

    test_channel
        .update_info(
            &mut bench,
            Some(ContentSource::String("hello".into())),
            &signing_authority,
        )
        .await;
}
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
