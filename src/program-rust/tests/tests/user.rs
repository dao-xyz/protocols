use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction, InstructionError},
    system_program,
};

use solana_program_test::*;
use solana_sdk::{account::Account, transaction::TransactionError, transport::TransportError};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solvei::{
    address::generate_seeds_from_string,
    social::{
        accounts::{
            deserialize_channel_account, deserialize_message_account, ChannelAccount, Description,
            Message,
        },
        instruction::{ChatInstruction, SendMessage},
    },
};

use crate::utils::program_test;

pub fn get_channel_account_address_and_bump_seed(
    channel_name: &str, // we should also send organization key,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let seeds = generate_seeds_from_string(channel_name).unwrap();
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Pubkey::find_program_address(seed_slice, program_id)
}

pub fn get_message_account_address_and_bump_seed(
    user_account: &Pubkey, // payer_account == from
    channel_account: &Pubkey,
    timestamp: &u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &user_account.to_bytes(),
            &channel_account.to_bytes(),
            &timestamp.to_le_bytes(),
        ],
        program_id,
    )
}

async fn create_and_verify_channel(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
    user_account: &Pubkey,
) -> Pubkey {
    let channel_name = "My channel";
    let (channel_address_pda, _bump) =
        get_channel_account_address_and_bump_seed(channel_name, program_id);

    let mut transaction_create = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            *program_id,
            &ChatInstruction::CreateChannel(ChannelAccount::new(
                *user_account,
                channel_name.into(),
                Description::String("This channel lets you channel channels".into()),
            )),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(*program_id, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(*user_account, false),
                AccountMeta::new(channel_address_pda, false),
            ], // WE SHOULD PASS PDA
        )],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[payer], *recent_blockhash);
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
    channel_address_pda
}

#[tokio::test]
async fn test_create_channel_send_message() {
    let program_id = Pubkey::new_unique();
    let program = program_test(program_id);
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let user_address_pda = crate::utils::create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &program_id,
    )
    .await;

    let channel_address_pda = create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &program_id,
        &user_address_pda,
    )
    .await;

    // Create a short message and submit it

    let first_message = "Hello world";

    let timestamp = 123_u64;
    let (message_account_pda, bump) = get_message_account_address_and_bump_seed(
        &user_address_pda,
        &channel_address_pda,
        &timestamp,
        &program_id,
    );
    let short_message = SendMessage {
        user: user_address_pda,
        channel: channel_address_pda,
        message: Message::String(first_message.into()),
        timestamp,
        bump_seed: bump,
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
                AccountMeta::new(user_address_pda, false),
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

    let _ = deserialize_channel_account(&channel_account_info.data);

    // Verify that that message contains expected data
    let message_account_info = banks_client
        .get_account(message_account_pda)
        .await
        .expect("get_account")
        .expect("message_account not found");
    let message_account = deserialize_message_account(&message_account_info.data);

    assert_eq!(
        message_account.message,
        Message::String(first_message.into())
    );
    assert_eq!(message_account.user, user_address_pda);
    assert_eq!(message_account.channel, channel_address_pda);
}

#[tokio::test]
async fn test_only_payer_can_be_user() {
    let program_id = Pubkey::new_unique();
    let mut program = program_test(program_id);
    let another_payer = Keypair::new();
    program.add_account(
        another_payer.pubkey(),
        Account {
            lamports: 11939600,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let user_address_pda = crate::utils::create_and_verify_user(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &program_id,
    )
    .await;
    let channel_address_pda = create_and_verify_channel(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &program_id,
        &user_address_pda,
    )
    .await;

    let first_message = "Hello world";

    // Create a short message and submit it

    let timestamp = 0_u64;
    let (message_account_pda, bump) = get_message_account_address_and_bump_seed(
        &user_address_pda,
        &channel_address_pda,
        &timestamp,
        &program_id,
    );
    let short_message = SendMessage {
        user: user_address_pda,
        channel: channel_address_pda,
        message: Message::String(first_message.into()),
        timestamp,
        bump_seed: bump,
    };

    // Create and submit message
    let mut transaction_message = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &ChatInstruction::SendMessage(short_message),
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(program_id, false),
                AccountMeta::new(another_payer.pubkey(), true),
                AccountMeta::new(user_address_pda, false),
                AccountMeta::new(channel_address_pda, false),
                AccountMeta::new(message_account_pda, false),
            ],
        )],
        Some(&another_payer.pubkey()),
    );
    transaction_message.sign(&[&another_payer], recent_blockhash);
    let error = banks_client
        .process_transaction(transaction_message)
        .await
        .err()
        .unwrap();

    assert!(matches!(
        error,
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::IllegalOwner
        ))
    ));
}

#[tokio::test]
async fn test_invalid_username() {
    let program_id = Pubkey::new_unique();
    let mut program = program_test(program_id);
    let another_payer = Keypair::new();
    program.add_account(
        another_payer.pubkey(),
        Account {
            lamports: 11939600,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let (transaction, _) =
        crate::utils::create_user_transaction(" x", &payer, &recent_blockhash, &program_id).await;
    let error = banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::InvalidArgument
        ))
    ));

    let (transaction, _) =
        crate::utils::create_user_transaction("x ", &payer, &recent_blockhash, &program_id).await;
    let error = banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::InvalidArgument
        ))
    ));
}
