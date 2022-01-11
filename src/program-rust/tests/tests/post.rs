use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    rent::Rent,
    system_program,
};

use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use solvei::{
    accounts::Message,
    instruction::{ChatInstruction, CreatePost, CreatePostContent, StakePost},
    processor::process,
};

use crate::utils::program_test;

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

#[tokio::test]
async fn test_create_post() {
    let program_id = Pubkey::new_unique();
    let program = program_test(program_id);
    let channel = Pubkey::new_unique();
    let timestamp = 123_u64;

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let user = crate::utils::create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &program_id,
    )
    .await;

    let (post_account_pda, post_bump_seed) = Pubkey::find_program_address(
        &[
            &user.to_bytes(),
            &channel.to_bytes(),
            &timestamp.to_le_bytes(),
        ],
        &program_id,
    );

    let (mint_account_pda, mint_bump_seed) =
        Pubkey::find_program_address(&["mint".as_bytes(), post_account_pda.as_ref()], &program_id);

    let (mint_authority_account_pda, mint_authority_bump_seed) =
        Pubkey::find_program_address(&[mint_account_pda.as_ref()], &program_id);

    let (user_post_token_account, user_post_token_account_bump_seed) = Pubkey::find_program_address(
        &[&user.to_bytes(), &post_account_pda.to_bytes()],
        &program_id,
    );

    let (escrow_account_info, escrow_account_bump_seed) = Pubkey::find_program_address(
        &["escrow".as_bytes(), &post_account_pda.to_bytes()],
        &program_id,
    );

    let message = Message::String("hello world".into());
    let hash = message.hash();

    let (post_content_account, post_content_account_bump_seed) =
        Pubkey::find_program_address(&[&hash], &program_id);

    let stake = 1000000000; // 1 sol

    // for testing purposes lets calcualte min rent
    let rent = Rent::default();
    let empty_account_minimum_balance = rent.minimum_balance(0);

    let mut transaction_post = Transaction::new_with_payer(
        &[
            Instruction::new_with_borsh(
                program_id,
                &ChatInstruction::CreatePost(CreatePost {
                    channel,
                    mint_bump_seed,
                    mint_authority_bump_seed,
                    spread_factor: None,
                    timestamp,
                    content: post_content_account,
                    post_bump_seed,
                    escrow_account_bump_seed,
                    user_post_token_account_bump_seed,
                }),
                vec![
                    AccountMeta::new(system_program::id(), false),
                    AccountMeta::new(program_id, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(user, false),
                    AccountMeta::new(post_account_pda, false),
                    AccountMeta::new(escrow_account_info, false),
                    AccountMeta::new(mint_account_pda, false),
                    AccountMeta::new(mint_authority_account_pda, false),
                    AccountMeta::new(user_post_token_account, false),
                    AccountMeta::new(solana_program::sysvar::rent::id(), false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
            ),
            Instruction::new_with_borsh(
                program_id,
                &ChatInstruction::CreatePostContent(CreatePostContent {
                    bump_seed: post_content_account_bump_seed,
                    message,
                }),
                vec![
                    AccountMeta::new(system_program::id(), false),
                    AccountMeta::new(program_id, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(post_content_account, false),
                ],
            ),
            Instruction::new_with_borsh(
                program_id,
                &ChatInstruction::StakePost(StakePost {
                    mint_authority_bump_seed,
                    user_post_token_account_bump_seed,
                    stake,
                    user,
                    post: post_account_pda,
                    escrow_account_bump_seed,
                }),
                vec![
                    AccountMeta::new(system_program::id(), false),
                    AccountMeta::new(program_id, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(post_account_pda, false),
                    AccountMeta::new(escrow_account_info, false),
                    AccountMeta::new(mint_account_pda, false),
                    AccountMeta::new(mint_authority_account_pda, false),
                    AccountMeta::new(user_post_token_account, false),
                    AccountMeta::new(solana_program::sysvar::rent::id(), false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
            ),
        ],
        Some(&payer.pubkey()),
    );

    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    let post_token_balance = get_token_balance(&mut banks_client, &user_post_token_account).await;
    assert_eq!(post_token_balance, stake + empty_account_minimum_balance); // stagnation factor non zero
    let escrow_balance = banks_client.get_balance(escrow_account_info).await.unwrap();
    assert_eq!(escrow_balance, stake + empty_account_minimum_balance);

    // Stake more
    let mut transaction_stake = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::StakePost(StakePost {
                mint_authority_bump_seed,
                user_post_token_account_bump_seed,
                stake,
                user,
                post: post_account_pda,
                escrow_account_bump_seed,
            }),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(post_account_pda, false),
                AccountMeta::new(escrow_account_info, false),
                AccountMeta::new(mint_account_pda, false),
                AccountMeta::new(mint_authority_account_pda, false),
                AccountMeta::new(user_post_token_account, false),
                AccountMeta::new(solana_program::sysvar::rent::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction_stake.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_stake)
        .await
        .unwrap();

    let new_expected_staked_amount = stake * 2;
    let post_token_balance = get_token_balance(&mut banks_client, &user_post_token_account).await;
    assert_eq!(
        post_token_balance,
        new_expected_staked_amount + empty_account_minimum_balance
    ); // stagnation factor non zero
    let escrow_balance = banks_client.get_balance(escrow_account_info).await.unwrap();
    assert_eq!(
        escrow_balance,
        new_expected_staked_amount + empty_account_minimum_balance
    );

    // Unstake
}
