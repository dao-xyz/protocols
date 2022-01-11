use solana_program::{
    borsh::{get_packed_len, try_from_slice_unchecked},
    instruction::{AccountMeta, Instruction},
    system_program,
};

use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solvei::{
    associated_token_account::get_associated_token_address_and_bump_seed,
    instruction::{ChatInstruction, InitializeStakePool, InitializeToken},
};
use spl_stake_pool::state::StakePool;

use crate::utils::{
    create_and_assign_program_owner_token, create_owner_token_account, program_test,
};

#[tokio::test]
async fn test_initialization() {
    // Test initialize the utility token (Solvei token) with dedicated owner (multisig)

    let program_id = Pubkey::new_unique();
    let owner = Keypair::new();
    let manager = Keypair::new();

    let mut program: ProgramTest = program_test(program_id);
    // Create an owner token mint
    create_owner_token_account(&mut program, &owner).await;
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    // This is the address that holds the owner token
    let owner_token_address =
        create_and_assign_program_owner_token(&mut banks_client, &payer, &recent_blockhash, &owner)
            .await;

    let (multisig_account_info, multisig_account_bump_seed) = Pubkey::find_program_address(
        &["token".as_bytes(), &owner.pubkey().to_bytes()],
        &program_id,
    );
    let (escrow_account_info, escrow_account_bump_seed) =
        Pubkey::find_program_address(&["escrow".as_bytes()], &program_id);
    let (mint_account_info, mint_account_bump_seed) =
        Pubkey::find_program_address(&["mint".as_bytes()], &program_id);
    let (stake_pool, stake_pool_bump_seed) =
        Pubkey::find_program_address(&["stake_pool".as_bytes()], &program_id);

    /* let (manager, manager_account_bump_seed) =
           Pubkey::find_program_address(&["manager".as_bytes(), &stake_pool.to_bytes()], &program_id);
    */
    let (staker, staker_bump_seed) =
        Pubkey::find_program_address(&["staker".as_bytes(), &stake_pool.to_bytes()], &program_id);

    // Calculate withdraw authority used for minting pool tokens
    let (withdraw_authority, withdraw_authority_bump_seed) =
        spl_stake_pool::find_withdraw_authority_program_address(&spl_stake_pool::id(), &stake_pool);

    let (validator_list, validator_list_bump_seed) = Pubkey::find_program_address(
        &["validator_list".as_bytes(), &stake_pool.to_bytes()],
        &program_id,
    );

    let (reserve_stake, reserve_stake_bump_seed) = Pubkey::find_program_address(
        &["reserve_stake".as_bytes(), &stake_pool.to_bytes()],
        &program_id,
    );

    let (manager_fee_account, manager_fee_account_bump_seed) =
        get_associated_token_address_and_bump_seed(
            &payer.pubkey(),
            &mint_account_info,
            &program_id,
        );

    let mut transaction_create = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::SetupStakePool(InitializeStakePool {
                // manager_bump_seed: manager_account_bump_seed,
                pool_mint_bump_seed: mint_account_bump_seed,
                reserve_stake_bump_seed,
                manager_fee_account_bump_seed,
                stake_pool_bump_seed: stake_pool_bump_seed,
                validator_list_bump_seed,
                stake_pool_packed_len: get_packed_len::<spl_stake_pool::state::StakePool>() as u64,
            }),
            vec![
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(owner.pubkey(), true),
                AccountMeta::new_readonly(owner_token_address, false),
                AccountMeta::new(stake_pool, false),
                AccountMeta::new(manager.pubkey(), true),
                AccountMeta::new(staker, false),
                AccountMeta::new(withdraw_authority, false),
                AccountMeta::new(validator_list, false),
                AccountMeta::new(reserve_stake, false),
                AccountMeta::new(mint_account_info, false),
                AccountMeta::new(manager_fee_account, false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(solana_program::stake::program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction_create.sign(&[&payer, &owner, &manager], recent_blockhash);
    banks_client
        .process_transaction(transaction_create)
        .await
        .unwrap();

    let mut transaction_initialize = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::InitializeStakePool(InitializeStakePool {
                //  manager_bump_seed: manager_account_bump_seed,
                pool_mint_bump_seed: mint_account_bump_seed,
                reserve_stake_bump_seed,
                manager_fee_account_bump_seed,
                stake_pool_bump_seed: stake_pool_bump_seed,
                validator_list_bump_seed,
                stake_pool_packed_len: get_packed_len::<spl_stake_pool::state::StakePool>() as u64,
            }),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(owner.pubkey(), true),
                AccountMeta::new(owner_token_address, false),
                AccountMeta::new(stake_pool, false),
                AccountMeta::new(manager.pubkey(), true),
                AccountMeta::new(staker, false),
                AccountMeta::new(withdraw_authority, false),
                AccountMeta::new(validator_list, false),
                AccountMeta::new(reserve_stake, false),
                AccountMeta::new(mint_account_info, false),
                AccountMeta::new(manager_fee_account, false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(solana_program::stake::program::id(), false),
                /*   AccountMeta::new_readonly(spl_stake_pool::id(), false), */
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction_initialize.sign(&[&payer, &owner, &manager], recent_blockhash);
    banks_client
        .process_transaction(transaction_initialize)
        .await
        .unwrap();
}
