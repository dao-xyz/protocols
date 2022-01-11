use solana_program::{
    account_info::{AccountInfo, IntoAccountInfo},
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};
use spl_associated_token_account::create_associated_token_account;
use std::str::FromStr;

use solana_program_test::*;

use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};
use solvei::{
    accounts::{deserialize_user_account, UserAccount},
    address::generate_seeds_from_string,
    associated_token_account::get_associated_token_address,
    instruction::ChatInstruction,
    owner::program_owner_token,
    processor::process,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::Mint,
};

pub fn program_test(program_id: Pubkey) -> ProgramTest {
    ProgramTest::new("solvei", program_id, processor!(process))
}

pub async fn create_owner_token_account(program: &mut ProgramTest, owner: &Keypair) -> Pubkey {
    let rent = Rent::default();
    let minimum_balance = rent.minimum_balance(Mint::LEN);

    let mut system_account = Account {
        lamports: minimum_balance,
        owner: spl_token::id(), // Can only withdraw lamports from accounts owned by the program
        data: vec![0; Mint::LEN],
        ..Account::default()
    };

    let mut mint = Mint::default();
    mint.mint_authority = COption::Some(owner.pubkey());
    mint.decimals = 0;
    mint.is_initialized = true;
    mint.freeze_authority = COption::None;

    Mint::pack(mint, &mut system_account.data).unwrap();
    program.add_account(program_owner_token::id(), system_account);
    program_owner_token::id()
}

pub async fn create_and_assign_program_owner_token(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    owner: &Keypair,
) -> Pubkey {
    // payer becomes the owner
    /* let mint = program_owner_token::id();
       let transaction_create = Transaction::new_with_payer(
           &[initialize_mint(
               &spl_token::id(),
               &mint,
               &payer.pubkey(),
               Some(&payer.pubkey()),
               0,
           )
           .unwrap()],
           Some(&payer.pubkey()),
       );
       banks_client
           .process_transaction(transaction_create)
           .await
           .unwrap();
    */
    let mint = program_owner_token::id();
    let transaction = Transaction::new_signed_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            &owner.pubkey(), // assume payer pubkey is also wallet address
            &mint,
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let associated_token_address = get_associated_token_address(&owner.pubkey(), &mint);

    let transaction_mint = Transaction::new_signed_with_payer(
        &[mint_to(
            &spl_token::id(),
            &mint,
            &associated_token_address,
            &owner.pubkey(),
            &[&owner.pubkey()],
            1,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[payer, owner],
        *recent_blockhash,
    );
    banks_client
        .process_transaction(transaction_mint)
        .await
        .unwrap();
    associated_token_address
}

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
