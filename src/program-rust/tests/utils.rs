use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    system_program,
};

use solana_program_test::*;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solvei::{
    accounts::{
        deserialize_user_account, UserAccount,
    },
    address::generate_seeds_from_string,
    instruction::{ChatInstruction},
};

pub fn get_user_account_address_and_bump_seed(username: &str, program_id: &Pubkey) -> (Pubkey, u8) {
    let seeds = generate_seeds_from_string(username).unwrap();
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Pubkey::find_program_address(seed_slice, program_id)
}

pub async fn create_user_transaction(
    username: &str,
    payer: &Keypair,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
) -> (Transaction, Pubkey) {
    let (user_address_pda, _) = get_user_account_address_and_bump_seed(username, program_id);

    let mut transaction_create = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            *program_id,
            &ChatInstruction::CreateUser(UserAccount {
                name: username.into(),
                owner: payer.pubkey(),
            }),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(*program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(user_address_pda, false),
            ], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );

    transaction_create.sign(&[payer], *recent_blockhash);
    (transaction_create, user_address_pda)
}

pub async fn create_and_verify_user(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
) -> Pubkey {
    // Create user
    let username = "Me";

    let (transaction, user_address_pda) =
        create_user_transaction(username, payer, recent_blockhash, program_id).await;
    banks_client
        .process_transaction(transaction)
        .await
        .unwrap_or_else(|err| {
            dbg!(err.to_string());
        });

    // Verify username name
    let user_account_info = banks_client
        .get_account(user_address_pda)
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data);
    assert_eq!(user.name, username);
    user_address_pda
}
