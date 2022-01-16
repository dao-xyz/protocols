#![cfg(feature = "test-bpf")]
use super::super::utils::program_test;

use {
    super::helpers::*,
    borsh::BorshSerialize,
    solana_program::{
        borsh::try_from_slice_unchecked,
        hash::Hash,
        instruction::{AccountMeta, Instruction},
    },
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError, signature::Keypair, signature::Signer,
        transaction::Transaction, transaction::TransactionError, transport::TransportError,
    },
    solvei::id,
    solvei::stake_pool::{
        error, find_deposit_authority_program_address,
        instruction::{self, FundingType},
        state,
    },
};

async fn setup() -> (BanksClient, Keypair, Hash, StakePoolAccounts, Keypair) {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let stake_pool_accounts = StakePoolAccounts::new();
    stake_pool_accounts
        .initialize_stake_pool(&mut banks_client, &payer, &recent_blockhash, 1)
        .await
        .unwrap();

    let new_deposit_authority = Keypair::new();

    (
        banks_client,
        payer,
        recent_blockhash,
        stake_pool_accounts,
        new_deposit_authority,
    )
}

#[tokio::test]
async fn success_set_stake_deposit_authority() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_authority) =
        setup().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            Some(&new_authority.pubkey()),
            FundingType::StakeDeposit,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(stake_pool.stake_deposit_authority, new_authority.pubkey());

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            None,
            FundingType::StakeDeposit,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(
        stake_pool.stake_deposit_authority,
        find_deposit_authority_program_address(&id(), &stake_pool_accounts.stake_pool).0
    );
}

#[tokio::test]
async fn fail_wrong_manager() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_authority) =
        setup().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &new_authority.pubkey(),
            Some(&new_authority.pubkey()),
            FundingType::StakeDeposit,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &new_authority], recent_blockhash);
    #[allow(clippy::useless_conversion)] // Remove during upgrade to 1.10
    let transaction_error = banks_client
        .process_transaction(transaction)
        .await
        .err()
        .unwrap()
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(error_index),
        )) => {
            let program_error = error::StakePoolError::WrongManager as u32;
            assert_eq!(error_index, program_error);
        }
        _ => panic!("Wrong error occurs while malicious try to set manager"),
    }
}

#[tokio::test]
async fn fail_without_signature() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_authority) =
        setup().await;

    let data = instruction::StakePoolInstruction::SetFundingAuthority(FundingType::StakeDeposit)
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new(stake_pool_accounts.stake_pool, false),
        AccountMeta::new_readonly(stake_pool_accounts.manager.pubkey(), false),
        AccountMeta::new_readonly(new_authority.pubkey(), false),
    ];
    let instruction = Instruction {
        program_id: id(),
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    #[allow(clippy::useless_conversion)] // Remove during upgrade to 1.10
    let transaction_error = banks_client
        .process_transaction(transaction)
        .await
        .err()
        .unwrap()
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(error_index),
        )) => {
            let program_error = error::StakePoolError::SignatureMissing as u32;
            assert_eq!(error_index, program_error);
        }
        _ => panic!("Wrong error occurs while try to set new manager without signature"),
    }
}

#[tokio::test]
async fn success_set_sol_deposit_authority() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_sol_deposit_authority) =
        setup().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            Some(&new_sol_deposit_authority.pubkey()),
            FundingType::SolDeposit,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(
        stake_pool.sol_deposit_authority,
        Some(new_sol_deposit_authority.pubkey())
    );

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            None,
            FundingType::SolDeposit,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(stake_pool.sol_deposit_authority, None);
}

#[tokio::test]
async fn success_set_withdraw_authority() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_authority) =
        setup().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            Some(&new_authority.pubkey()),
            FundingType::SolWithdraw,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(
        stake_pool.sol_withdraw_authority,
        Some(new_authority.pubkey())
    );

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            None,
            FundingType::SolWithdraw,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(stake_pool.sol_withdraw_authority, None);
}
