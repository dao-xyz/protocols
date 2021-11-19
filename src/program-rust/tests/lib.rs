
use solvei::{ChannelAccount, ChatInstruction, MessageAccount, NULL_KEY, address::{get_channel_account_address_and_bump_seed,get_message_account_address_and_bump_seed}};
use solana_program::{instruction::{AccountMeta, Instruction}, system_program};
use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction, borsh::try_from_slice_unchecked};
mod test_program;


#[tokio::test]
async fn test_create_channel_send_message() {

    let program_id = Pubkey::new_unique();
    let program = test_program::program_test(program_id);
    let channel_name = "My channel";
    let (channel_address_pda, bump) = get_channel_account_address_and_bump_seed(channel_name,&program_id);
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    
    
    // Create channel
    let mut transaction_create = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::CreateChannel(ChannelAccount::new(channel_name.into())), 
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(channel_address_pda, false)], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction_create).await.unwrap(); 

    // Verify channel name
    let channel_account = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account_data:ChannelAccount =  try_from_slice_unchecked(&channel_account.data)
    .unwrap();
    
    assert_eq!(
        channel_account_data.name.as_str(),
        channel_name
    ); 

    assert_eq!(
        channel_account_data.tail_message,
        NULL_KEY
    ); 

    // Create a short message and submit it
    let short_message = MessageAccount::new_short_message("Hello world".into(), payer.pubkey());
    let (message_account_pda, bump) = get_message_account_address_and_bump_seed(&payer.pubkey(),&program_id);

    // Create and submit message
    let mut transaction_message = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::BuildMessageInitialize(short_message), 
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(message_account_pda, false), AccountMeta::new(channel_address_pda, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction_message.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction_message).await.unwrap(); 


    // Read message from channel tail
    /* let channel_account = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account_data:ChannelAccount =  try_from_slice_unchecked(&channel_account.data)
    .unwrap();
    
    assert_eq!(channel_account_data.tail_message, message_account_pda); */

    /* 
    // Update channel 
    let mut updated_channel = ChannelAccount::new(channel_name.into());
    let new_tail_message_key = Pubkey::new_unique();
    updated_channel.tail_message = new_tail_message_key;

    let mut transaction_update = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::UpdateChannel(updated_channel), 
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(channel_address_pda, false)], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );
    transaction_update.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction_update).await.unwrap(); 

    // Verify new tail message key
    let channel_account = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let account_data:ChannelAccount =  try_from_slice_unchecked(&channel_account.data).unwrap();
    
    // Check name has not changed
    assert_eq!(
        account_data.name.as_str(),
        channel_name
    );   

    // Check tail s is some new key
    assert_eq!(
        account_data.tail_message,
        new_tail_message_key
    );    */




}
