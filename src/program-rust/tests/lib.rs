#![cfg(feature = "test-bpf")]

use borsh::BorshDeserialize;
use solvei::{ChannelAccount, ChatInstruction, address::{get_channel_address,get_channel_address_and_bump_seed}, process_instruction};
use solana_program::{instruction::{AccountMeta, Instruction}, system_program};
use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::mem;
mod test_program;

#[tokio::test]
async fn test_create_update_channel() {
    let program_id = Pubkey::new_unique();
    let mut program = test_program::program_test(program_id);
    let channel_name = "Name";
    let (channel_address_pda, bump) = get_channel_address_and_bump_seed(channel_name,&program_id);
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    
    
    // Create channel
    let instruction = ChatInstruction::CreateChannel(ChannelAccount::new(channel_name.into()));
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction, 
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(channel_address_pda, false)], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap(); 

    // Verify channel name
    let channel_account = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let account_data =  ChannelAccount::try_from_slice(&channel_account.data)
    .unwrap();
    
    assert_eq!(
        account_data.name.as_str(),
        "c"
    ); 


    // Update channel
    /* let updated_channel_name = "A looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong name";
    let instruction = ChatInstruction::UpdateChannel(ChannelAccount::new(updated_channel_name.into()));
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction, 
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(channel_address_pda, false)], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap(); 

    // Verify channel name
    let channel_account = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let account_data =  ChannelAccount::try_from_slice(&channel_account.data)
    .unwrap();
    
    assert_eq!(
        account_data.name.as_str(),
        updated_channel_name
    );  */



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
