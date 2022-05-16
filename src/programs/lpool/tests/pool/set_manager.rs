#![cfg(feature = "test-bpf")]
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};

use super::super::utils::program_test;

use {
    super::helpers::*,
    borsh::BorshSerialize,
    lpool::id,
    lpool::{error, instruction, state},
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
};

async fn setup() -> (
    BanksClient,
    Keypair,
    Hash,
    StakePoolAccounts,
    Keypair,
    Keypair,
) {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let stake_pool_accounts = StakePoolAccounts::new();
    stake_pool_accounts
        .initialize_stake_pool(&mut banks_client, &payer, &recent_blockhash, 1)
        .await
        .unwrap();

    let new_pool_fee = Keypair::new();
    let new_manager = Keypair::new();
    create_token_account(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &new_pool_fee,
        &stake_pool_accounts.pool_mint,
        &new_manager.pubkey(),
    )
    .await
    .unwrap();

    (
        banks_client,
        payer,
        recent_blockhash,
        stake_pool_accounts,
        new_pool_fee,
        new_manager,
    )
}

pub async fn create_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    pool_mint: &Keypair,
    manager: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &pool_mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &pool_mint.pubkey(),
                manager,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, pool_mint], *recent_blockhash);
    #[allow(clippy::useless_conversion)] // Remove during upgrade to 1.10
    banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

#[tokio::test]
async fn test_set_manager() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_pool_fee, new_manager) =
        setup().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_manager(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            &new_manager.pubkey(),
            &new_pool_fee.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &[&payer, &stake_pool_accounts.manager, &new_manager],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_pool = get_account(&mut banks_client, &stake_pool_accounts.stake_pool).await;
    let stake_pool =
        try_from_slice_unchecked::<state::StakePool>(stake_pool.data.as_slice()).unwrap();

    assert_eq!(stake_pool.manager, new_manager.pubkey());
}

#[tokio::test]
async fn test_set_manager_by_malicious() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_pool_fee, new_manager) =
        setup().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_manager(
            &id(),
            &stake_pool_accounts.stake_pool,
            &new_manager.pubkey(),
            &new_manager.pubkey(),
            &new_pool_fee.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &new_manager], recent_blockhash);
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
async fn test_set_manager_without_existing_signature() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_pool_fee, new_manager) =
        setup().await;

    let data = instruction::StakePoolInstruction::SetManager
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new(stake_pool_accounts.stake_pool, false),
        AccountMeta::new_readonly(stake_pool_accounts.manager.pubkey(), false),
        AccountMeta::new_readonly(new_manager.pubkey(), true),
        AccountMeta::new_readonly(new_pool_fee.pubkey(), false),
    ];
    let instruction = Instruction {
        program_id: id(),
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &new_manager], recent_blockhash);
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
        _ => panic!(
            "Wrong error occurs while try to set new manager without existing manager signature"
        ),
    }
}

#[tokio::test]
async fn test_set_manager_without_new_signature() {
    let (mut banks_client, payer, recent_blockhash, stake_pool_accounts, new_pool_fee, new_manager) =
        setup().await;

    let data = instruction::StakePoolInstruction::SetManager
        .try_to_vec()
        .unwrap();
    let accounts = vec![
        AccountMeta::new(stake_pool_accounts.stake_pool, false),
        AccountMeta::new_readonly(stake_pool_accounts.manager.pubkey(), true),
        AccountMeta::new_readonly(new_manager.pubkey(), false),
        AccountMeta::new_readonly(new_pool_fee.pubkey(), false),
    ];
    let instruction = Instruction {
        program_id: id(),
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &stake_pool_accounts.manager], recent_blockhash);
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
        _ => {
            panic!("Wrong error occurs while try to set new manager without new manager signature")
        }
    }
}

#[tokio::test]
async fn test_set_manager_with_wrong_mint_for_pool_fee_acc() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let stake_pool_accounts = StakePoolAccounts::new();
    stake_pool_accounts
        .initialize_stake_pool(&mut banks_client, &payer, &recent_blockhash, 1)
        .await
        .unwrap();

    let new_mint = Keypair::new();
    let new_withdraw_auth = Keypair::new();
    let new_pool_fee = Keypair::new();
    let new_manager = Keypair::new();

    create_mint(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &new_mint,
        &new_withdraw_auth.pubkey(),
    )
    .await
    .unwrap();
    create_token_account(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &new_pool_fee,
        &new_mint.pubkey(),
        &new_manager.pubkey(),
    )
    .await
    .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[instruction::set_manager(
            &id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.manager.pubkey(),
            &new_manager.pubkey(),
            &new_pool_fee.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &[&payer, &stake_pool_accounts.manager, &new_manager],
        recent_blockhash,
    );
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
            let program_error = error::StakePoolError::WrongAccountMint as u32;
            assert_eq!(error_index, program_error);
        }
        _ => panic!("Wrong error occurs while try to set new manager with wrong mint"),
    }
}
