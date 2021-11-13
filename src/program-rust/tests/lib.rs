use borsh::{BorshSerialize};
use helloworld::{ChannelAccount, ChatInstruction, process_instruction};
use solana_program::system_program;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::mem;

#[tokio::test]
async fn test_helloworld() {
    let program_id = Pubkey::new_unique();
    let domain_id = Pubkey::new_unique();

    let x = program_id.to_string();
    let mut program_test = ProgramTest::new(
        "helloworld", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );

    println!("{}", program_id.to_string());
    
    program_test.add_account(
        domain_id,
        Account {
            lamports: 5,
            data: vec![0_u8; mem::size_of::<u32>()],
            owner: domain_id,
            ..Account::default()
        },
    );



    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Verify account has zero greetings
    /*let organization_account = banks_client
        .get_account(organization_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(
        OrganizationAccount::try_from_slice(&organization_account.data)
            .unwrap()
            .counter,
        0
    );*/

    // Greet once
    let instruction = ChatInstruction::CreateChannel(ChannelAccount::new("org".into()));
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction, // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(domain_id, true), AccountMeta::new(payer.pubkey(), true)],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

}
 /* 
#[tokio::test]
async fn test_helloworld() {
    let program_id = Pubkey::new_unique();
    let organization_pubkey = Pubkey::new_unique();

    let mut program_test = ProgramTest::new(
        "helloworld", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );
    program_test.add_account(
        organization_pubkey,
        Account {
            lamports: 5,
            data: vec![0_u8; mem::size_of::<u32>()],
            owner: program_id,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Verify account has zero greetings
    let organization_account = banks_client
        .get_account(organization_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(
        OrganizationAccount::try_from_slice(&organization_account.data)
            .unwrap()
            .counter,
        0
    );

    // Greet once
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(greeted_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify account has one greeting
    let greeted_account = banks_client
        .get_account(greeted_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(
        GreetingAccount::try_from_slice(&greeted_account.data)
            .unwrap()
            .counter,
        1
    );

    // Greet again
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[1], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(greeted_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify account has two greetings
    let greeted_account = banks_client
        .get_account(greeted_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(
        GreetingAccount::try_from_slice(&greeted_account.data)
            .unwrap()
            .counter,
        2
    );
}*/
