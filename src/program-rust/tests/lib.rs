
use solvei::{ChannelAccount, ChatInstruction, Message, MessageAccount, NULL_KEY, address::{get_channel_account_address_and_bump_seed,get_message_account_address_and_bump_seed}};
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
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account:ChannelAccount =  try_from_slice_unchecked(&channel_account_info.data)
    .unwrap();
    
    assert_eq!(
        channel_account.name.as_str(),
        channel_name
    ); 

    assert_eq!(
        channel_account.tail_message,
        NULL_KEY
    ); 

    let first_message = "Hello world";

    // Create a short message and submit it
    let short_message = MessageAccount::new_message(Message::String(first_message.into()), payer.pubkey());
    let (message_account_pda, _) = get_message_account_address_and_bump_seed(&payer.pubkey(),&program_id);

    // Create and submit message
    let mut transaction_message = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::SendMessage(short_message), 
            vec![AccountMeta::new(system_program::id(), false), AccountMeta::new(program_id, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(message_account_pda, false), AccountMeta::new(channel_address_pda, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction_message.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction_message).await.unwrap(); 

    // Verify channel has updated reference to last message
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account:ChannelAccount =  try_from_slice_unchecked(&channel_account_info.data)
    .unwrap();
    assert_eq!(channel_account.tail_message,message_account_pda);

     // Verify that that message contains expected data
    let message_account_info = banks_client
        .get_account(message_account_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let message_account:MessageAccount =  try_from_slice_unchecked(&message_account_info.data)
    .unwrap();
    
    assert_eq!(message_account.message, Message::String(first_message.into()));
    assert_eq!(message_account.from, payer.pubkey());

    // Lets try to send a bigger message that contain multiple parts




}
