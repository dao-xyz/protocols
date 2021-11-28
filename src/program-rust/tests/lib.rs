use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use solvei::{ChatInstruction, SendMessage, accounts::{AccountContainer, ChannelAccount, Message, MessageAccount, deserialize_channel_account, deserialize_message_account}, address::{
        get_channel_account_address_and_bump_seed, get_message_account_address_and_bump_seed,
    }};
mod test_program;
#[tokio::test]
async fn test_create_channel_send_message() {
    let program_id = Pubkey::new_unique();
    let program = test_program::program_test(program_id);
    let channel_name = "My channel";
    let (channel_address_pda, bump) =
        get_channel_account_address_and_bump_seed(channel_name, &program_id);
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    // Create channel
    let mut transaction_create = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::CreateChannel(ChannelAccount::new(channel_name.into())),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(channel_address_pda, false),
            ], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_create)
        .await
        .unwrap();

    // Verify channel name
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account = deserialize_channel_account(&channel_account_info.data);

    assert_eq!(channel_account.name.as_str(), channel_name);

    let first_message = "Hello world";

    // Create a short message and submit it
    
    let timestamp = 0;
    let (message_account_pda,bump) = get_message_account_address_and_bump_seed(
        &payer.pubkey(),
        &channel_address_pda,
        &timestamp,
        &program_id,
    );
    let short_message = SendMessage { 
        message: Message::String(first_message.into()),
        timestamp,
        channel: channel_address_pda,
        bump_seed: bump
    };

    // Create and submit message
    let mut transaction_message = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::SendMessage(short_message),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(channel_address_pda, false),
                AccountMeta::new(message_account_pda, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction_message.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_message)
        .await
        .unwrap();

    // Verify channel has updated reference to last message
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account =  deserialize_channel_account(&channel_account_info.data);


    // Verify that that message contains expected data
    let message_account_info = banks_client
        .get_account(message_account_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let message_account = deserialize_message_account(&message_account_info.data);

    assert_eq!(
        message_account.message,
        Message::String(first_message.into())
    );
    assert_eq!(message_account.from, payer.pubkey());

    // Lets try to send a bigger message that contain multiple parts
}
