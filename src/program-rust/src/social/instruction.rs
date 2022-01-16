use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{
    instruction::CHAT_INSTRUCTION_INDEX,
    shared::io_utils::try_to_vec_prepend,
    social::accounts::{ChannelAccount, Message, UserAccount},
    tokens::spl_utils::{find_mint_authority_program_address, find_mint_escrow_program_address},
};

use super::{
    accounts::Description, find_channel_program_address, find_post_content_program_address,
    find_post_mint_program_address, find_post_program_address, find_user_account_program_address,
    find_user_post_token_program_address,
};
/*
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct SendMessage {
    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub message: Message,
    pub bump_seed: u8,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct SubmitMessage {
    pub from: Pubkey,
}*/

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePost {
    pub channel: Pubkey,
    pub timestamp: u64,
    pub spread_factor: Option<u64>,
    pub content: Pubkey,
    pub post_bump_seed: u8,
    pub mint_escrow_bump_seed: u8,
    pub mint_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
    pub user_post_token_account_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePostContent {
    pub message: Message,
    pub bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct StakePost {
    pub user: Pubkey,
    pub post: Pubkey,
    pub stake: u64,
    pub user_post_token_account_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
    pub mint_escrow_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ChatInstruction {
    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    CreateUser(UserAccount),

    // Create channel
    CreateChannel(ChannelAccount),

    // Update channel (the tail message)
    UpdateChannel(ChannelAccount),

    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    // SendMessage(SendMessage),
    // Add message to message builder
    //BuildMessagePart(String),

    // Submit message from BuildMessage invocations
    //SubmitMessage,
    CreatePost(CreatePost),

    /// Must be included in the same transaction as the CreatePost transaction
    CreatePostContent(CreatePostContent),

    // "Like" the post with an amount
    StakePost(StakePost),
}

impl ChatInstruction {
    /**
     * Prepends global instruction index
     */
    pub fn try_to_vec(&self) -> Result<Vec<u8>> {
        try_to_vec_prepend(CHAT_INSTRUCTION_INDEX, self)
    }
}

/// Creates a create user transction
pub fn create_user_transaction(program_id: &Pubkey, username: &str, payer: &Pubkey) -> Instruction {
    let (user_address_pda, _) = find_user_account_program_address(program_id, username);
    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreateUser(UserAccount {
            name: username.into(),
            owner: *payer,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new(*payer, true),
            AccountMeta::new(user_address_pda, false),
        ],
    }
}

/// Creates a create user transction
pub fn create_channel_transaction(
    program_id: &Pubkey,
    channel_name: &str,
    payer: &Pubkey,
    user: &Pubkey,
) -> Instruction {
    let (channel, _) = find_channel_program_address(program_id, channel_name);

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreateChannel(ChannelAccount {
            name: channel_name.into(),
            description: Description::String("This channel lets you channel channels".into()),
            owner: *payer,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new(*payer, true),
            AccountMeta::new(*user, false),
            AccountMeta::new(channel, false),
        ],
    }
}

/// Creates a create post transction
pub fn create_post_transaction(
    program_id: &Pubkey,
    channel: &Pubkey,
    payer: &Pubkey,
    user: &Pubkey,
    timestamp: u64,
) -> Instruction {
    let (post_address, post_bump_seed) =
        find_post_program_address(program_id, user, &channel, timestamp);
    let (post_content_address, _) = find_post_content_program_address(program_id, &post_address);

    let (mint_address, mint_bump_seed) = find_post_mint_program_address(program_id, &post_address);
    let q = mint_address.as_ref();
    let q2 = &mint_address.to_bytes();
    assert_eq!(q, q2);
    let (mint_authority_address, mint_authority_bump_seed) =
        find_mint_authority_program_address(program_id, &mint_address);
    let (mint_escrow_address, mint_escrow_bump_seed) =
        find_mint_escrow_program_address(program_id, &mint_address);
    let (user_post_token_account, user_post_token_account_bump_seed) =
        find_user_post_token_program_address(program_id, &post_address, user);
    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreatePost(CreatePost {
            channel: *channel,
            mint_bump_seed,
            mint_authority_bump_seed,
            spread_factor: None,
            timestamp,
            content: post_content_address,
            post_bump_seed,
            mint_escrow_bump_seed,
            user_post_token_account_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new(*payer, true),
            AccountMeta::new(*user, false),
            AccountMeta::new(post_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(mint_authority_address, false),
            AccountMeta::new(mint_escrow_address, false),
            AccountMeta::new(user_post_token_account, false),
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    }
}

/// Creates a create post content transction.
pub fn create_post_content_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    message: Message,
) -> Instruction {
    let (post_content_address, post_content_bump_seed) =
        find_post_content_program_address(program_id, &post);

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreatePostContent(CreatePostContent {
            bump_seed: post_content_bump_seed,
            message,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new(*payer, true),
            AccountMeta::new(post_content_address, false),
        ],
    }
}

// "Stake" solvei tokens
pub fn create_post_stake_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    user: &Pubkey,
    post: &Pubkey,
    stake: u64,
) -> Instruction {
    let (post_content_address, _) = find_post_content_program_address(program_id, post);
    let (mint_address, mint_bump_seed) = find_post_mint_program_address(program_id, user);
    let (mint_authority_address, mint_authority_bump_seed) =
        find_mint_authority_program_address(program_id, user);
    let (mint_escrow_address, mint_escrow_bump_seed) =
        find_mint_escrow_program_address(program_id, user);
    let (user_post_token_account, user_post_token_account_bump_seed) =
        find_user_post_token_program_address(program_id, &post, user);

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::StakePost(StakePost {
            mint_authority_bump_seed,
            user_post_token_account_bump_seed,
            stake,
            user: *user,
            post: post_content_address,
            mint_escrow_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new(*payer, true),
            AccountMeta::new(*post, false),
            AccountMeta::new(mint_escrow_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(mint_authority_address, false),
            AccountMeta::new(user_post_token_account, false),
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    }
}
