use solana_program::borsh::try_from_slice_unchecked;
use solana_program::hash::Hash;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use solana_sdk::{transaction::TransactionError, transport::TransportError};
use westake::socials::find_user_account_program_address;
use westake::socials::instruction::create_user_transaction;
use westake::socials::state::{deserialize_user_account, AccountContainer, UserAccount};

use crate::utils::program_test;

pub async fn create_and_verify_user(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    username: &str,
) -> Pubkey {
    // Create user
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &westake::id(),
                username,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify username name
    let user_account_address = find_user_account_program_address(&westake::id(), username).0;
    let user_account_info = banks_client
        .get_account(user_account_address.clone())
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data).unwrap();
    assert_eq!(user.name, username);
    user_account_address
}

#[tokio::test]
async fn success() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    create_and_verify_user(&mut banks_client, &payer, &recent_blockhash, "name").await;
}

#[tokio::test]
async fn fail_invalid_username() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let error = banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &westake::id(),
                " x",
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
                &westake::id(),
                "x ",
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
