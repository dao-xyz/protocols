use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};

use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solvei::instruction::{ChatInstruction, InitializeToken};

use crate::utils::{
    create_and_assign_program_owner_token, create_owner_token_account, program_test,
};

#[tokio::test]
async fn test_initialization() {
    // Test initialize the utility token (Solvei token) with dedicated owner (multisig)
    let program_id = Pubkey::new_unique();
    let owner = Keypair::new();
    let mut program = program_test(program_id);

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
        Pubkey::find_program_address(&["escrow".as_bytes(), &program_id.to_bytes()], &program_id);
    let (mint_account_info, mint_account_bump_seed) =
        Pubkey::find_program_address(&["mint".as_bytes(), &program_id.to_bytes()], &program_id);

    let mut transaction_create = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::InitializeToken(InitializeToken {
                escrow_bump_seed: escrow_account_bump_seed,
                mint_bump_seed: mint_account_bump_seed,
                multisig_bump_seed: multisig_account_bump_seed,
            }),
            vec![
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(owner.pubkey(), true),
                AccountMeta::new(escrow_account_info, false),
                AccountMeta::new(mint_account_info, false),
                AccountMeta::new(multisig_account_info, false),
                AccountMeta::new_readonly(owner_token_address, false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction_create.sign(&[&payer, &owner], recent_blockhash);
    banks_client
        .process_transaction(transaction_create)
        .await
        .unwrap();
}
