use solana_program::{
    account_info::{AccountInfo, IntoAccountInfo},
    entrypoint::ProgramResult,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_error::PrintProgramError,
    program_option::COption,
    program_pack::Pack,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use std::str::FromStr;

use solana_program_test::*;

use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::Mint,
};
use westake::{
    id,
    owner::program_owner_token,
    processor::Processor,
    social::{
        accounts::{deserialize_user_account, UserAccount},
        find_user_account_program_address,
        instruction::{create_user_transaction, ChatInstruction},
    },
};

fn swap_pool_process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //Processor::process(program_id, accounts, instruction_data)
    if let Err(error) =
        spl_token_swap::processor::Processor::process(program_id, accounts, instruction_data)
    {
        // catch the error so we can print it
        error.print::<spl_token_swap::error::SwapError>();
        Err(error)
    } else {
        Ok(())
    }
}

pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new("westake", westake::id(), processor!(Processor::process));
    program.prefer_bpf(false);
    program.add_program(
        "spl_token_swap",
        spl_token_swap::id(),
        processor!(swap_pool_process_instruction),
    );
    program
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

pub async fn create_and_verify_user(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    username: &str,
) -> Pubkey {
    // Create user
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(&id(), username, &payer.pubkey())],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify username name
    let user_account_address = find_user_account_program_address(&id(), username).0;
    let user_account_info = banks_client
        .get_account(user_account_address.clone())
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data);
    assert_eq!(user.name, username);
    user_account_address
}
