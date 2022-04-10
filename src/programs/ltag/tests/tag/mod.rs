use ltag::state::{TagAccount, TagRecordAccount, TagRecordFactoryAccount};
use ltag::{
    get_tag_program_address, get_tag_record_factory_program_address, get_tag_record_program_address,
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::hash::Hash;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};

use crate::utils::program_test;

pub async fn create_tag(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    tag: &str,
    authority: &Keypair,
) -> Pubkey {
    // Create tag
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ltag::instruction::create_tag(
                &ltag::id(),
                tag,
                None,
                &authority.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer, authority],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    let tag_address = get_tag_program_address(&ltag::id(), tag).0;
    let tag_account_info = banks_client
        .get_account(tag_address)
        .await
        .expect("get_tag")
        .expect("tag not found");
    let tag_account = try_from_slice_unchecked::<TagAccount>(&tag_account_info.data).unwrap();
    assert_eq!(tag_account.tag, tag);
    tag_address
}

pub async fn create_tag_record(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    tag: &Pubkey,
    owner: &Pubkey,
    factory: &Pubkey,
    authority: &Keypair,
) -> Pubkey {
    // Create tag record
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ltag::instruction::create_tag_record(
                &ltag::id(),
                tag,
                owner,
                factory,
                &authority.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer, authority],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    let tag_record_address = get_tag_record_program_address(&ltag::id(), factory, owner).0;
    let tag_record_account_info = banks_client
        .get_account(tag_record_address)
        .await
        .expect("get_tag_record")
        .expect("tag not found");
    let tag_record_account =
        try_from_slice_unchecked::<TagRecordAccount>(&tag_record_account_info.data).unwrap();
    assert_eq!(&tag_record_account.factory, factory);
    assert_eq!(&tag_record_account.owner, owner);
    assert_eq!(&tag_record_account.tag, tag);
    tag_record_address
}

pub async fn delete_tag_record(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    tag_record: &Pubkey,
    factory: &Pubkey,
    authority: &Keypair,
    owner: &Keypair,
    withdraw_destination: &Pubkey,
) {
    // Delete tag record
    let balance_pre = banks_client
        .get_balance(*withdraw_destination)
        .await
        .unwrap();
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ltag::instruction::delete_tag_record_as_owner(
                &ltag::id(),
                tag_record,
                &owner.pubkey(),
                factory,
                &authority.pubkey(),
                withdraw_destination,
            )],
            Some(&payer.pubkey()),
            &[payer, owner],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    assert_eq!(
        banks_client
            .get_account(*tag_record)
            .await
            .expect("get_tag_record")
            .is_none(),
        true
    );

    let balance_post = banks_client
        .get_balance(*withdraw_destination)
        .await
        .unwrap();
    assert!(balance_pre < balance_post); // Redeemed some lamports
}

pub async fn create_tag_record_factory(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    tag: &Pubkey,
    authority: &Keypair,
) -> Pubkey {
    // Create tag record
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ltag::instruction::create_tag_record_factory(
                &ltag::id(),
                tag,
                &authority.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer, authority],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    let tag_record_factory_address =
        get_tag_record_factory_program_address(&ltag::id(), tag, &authority.pubkey()).0;
    let tag_record_factory_account_info = banks_client
        .get_account(tag_record_factory_address)
        .await
        .expect("get_tag_record_factory")
        .expect("tag not found");
    let tag_record_factory_account =
        try_from_slice_unchecked::<TagRecordFactoryAccount>(&tag_record_factory_account_info.data)
            .unwrap();
    assert_eq!(&tag_record_factory_account.authority, &authority.pubkey());
    assert_eq!(&tag_record_factory_account.outstanding_records, &0);
    assert_eq!(&tag_record_factory_account.tag, tag);
    tag_record_factory_address
}

async fn get_outstanding_records(banks_client: &mut BanksClient, factory: &Pubkey) -> u64 {
    let tag_record_factory_account_info = banks_client
        .get_account(*factory)
        .await
        .expect("get_tag_record_factory")
        .expect("tag not found");
    let tag_record_factory_account =
        try_from_slice_unchecked::<TagRecordFactoryAccount>(&tag_record_factory_account_info.data)
            .unwrap();
    tag_record_factory_account.outstanding_records
}
#[tokio::test]
async fn success() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let tag = "name";
    let tag_authority = Keypair::new();

    let tag = create_tag(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        tag,
        &tag_authority,
    )
    .await;

    let owner = Keypair::new();
    let factory_authority = Keypair::new();
    let factory = create_tag_record_factory(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &tag,
        &factory_authority,
    )
    .await;

    let tag_reccord = create_tag_record(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &tag,
        &owner.pubkey(),
        &factory,
        &factory_authority,
    )
    .await;

    assert_eq!(
        get_outstanding_records(&mut banks_client, &factory).await,
        1
    );

    delete_tag_record(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &tag_reccord,
        &factory,
        &factory_authority,
        &owner,
        &payer.pubkey(),
    )
    .await;

    assert_eq!(
        get_outstanding_records(&mut banks_client, &factory).await,
        0
    );
}
/*
#[tokio::test]
async fn fail_update_wrong_payer() {
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
    create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        username,
        "profile",
    )
    .await;

    let profile = "updated_profile";
    let err = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_update_user_transaction(
                &luser::id(),
                username,
                Some(profile.into()),
                &wrong_payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &wrong_payer],
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
    create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        username,
        "profile",
    )
    .await;

    let profile = "updated_profile";
    let (user_account, _) = find_user_account_program_address(&luser::id(), username);

    let err = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: luser::id(),
                data: (UserInstruction::UpdateUser {
                    profile: Some(profile.into()),
                })
                .try_to_vec()
                .unwrap(),
                accounts: vec![
                    AccountMeta::new(wrong_payer.pubkey(), false),
                    AccountMeta::new(user_account, false),
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

#[tokio::test]
async fn fail_invalid_username() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let error = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                " x",
                None,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        ))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::InvalidArgument
        ))
    ));

    let error = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                "x ",
                None,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        ))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::InvalidArgument
        ))
    ));
}

#[tokio::test]
async fn fail_already_exist() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                "X",
                None,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        ))
        .await
        .unwrap();

    let error = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                "x",
                None,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        ))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::InvalidArgument
        ))
    ));
}
 */
