use luser::find_user_account_program_address;
use luser::instruction::{
    create_update_user_transaction, create_user_transaction, UserInstruction,
};
use luser::state::deserialize_user_account;
use solana_program::hash::Hash;
use solana_program::instruction::{AccountMeta, Instruction, InstructionError};
use solana_program_test::*;

use solana_sdk::signature::Keypair;
use solana_sdk::transaction::TransactionError;
use solana_sdk::transport::TransportError;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};

use crate::utils::program_test;

use borsh::ser::BorshSerialize;

pub async fn create_and_verify_user(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    username: &str,
    profile: &str,
    owner: &Keypair,
) -> Pubkey {
    // Create user
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                username,
                Some(profile.into()),
                &owner.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer, owner],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify username name
    let user_account_address = find_user_account_program_address(&luser::id(), username).0;
    let user_account_info = banks_client
        .get_account(user_account_address)
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data).unwrap();
    assert_eq!(user.name, username);
    user_account_address
}

#[tokio::test]
async fn success_create() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        "name",
        "profile",
        &Keypair::new(),
    )
    .await;
}

#[tokio::test]
async fn success_update() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let username = "name";
    let owner = Keypair::new();
    create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        username,
        "ipfs://kkk",
        &owner,
    )
    .await;

    let profile =
        "ipfs://kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_update_user_transaction(
                &luser::id(),
                username,
                Some(profile.into()),
                &owner.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &owner],
            recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify profile changed
    let user_account_address = find_user_account_program_address(&luser::id(), username).0;
    let user_account_info = banks_client
        .get_account(user_account_address)
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data).unwrap();
    assert_eq!(user.profile, Some(profile.into()));
}

#[tokio::test]
async fn fail_update_wrong_owner() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let owner = Keypair::new();

    let username = "name";
    create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        username,
        "profile",
        &owner,
    )
    .await;

    let profile = "updated_profile";
    let wrong_owner = Keypair::new();
    let err = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_update_user_transaction(
                &luser::id(),
                username,
                Some(profile.into()),
                &wrong_owner.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &wrong_owner],
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
async fn fail_invalid_username() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let owner = Keypair::new();
    let error = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                " x",
                None,
                &owner.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &owner],
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
                &owner.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &owner],
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
    let owner = Keypair::new();
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                "X",
                None,
                &owner.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &owner],
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
                &owner.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[&payer, &owner],
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
