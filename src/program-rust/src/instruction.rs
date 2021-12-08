use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::accounts::{ChannelAccount, Message, UserAccount};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SendMessage {
    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub message: Message,
    pub bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SubmitMessage {
    pub from: Pubkey,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreatePost {
    pub channel: Pubkey,
    pub timestamp: u64,
    pub spread_factor: Option<u64>,
    pub content: Pubkey,
    pub post_bump_seed: u8,
    pub escrow_account_bump_seed: u8,
    pub mint_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
    pub user_post_token_account_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreatePostContent {
    pub message: Message,
    pub bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct StakePost {
    pub user: Pubkey,
    pub post: Pubkey,
    pub stake: u64,
    pub user_post_token_account_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
    pub escrow_account_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ChatInstruction {
    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    CreateUser(UserAccount),

    // Create channel, that keep tracks of the message tail
    CreateChannel(ChannelAccount),

    // Update channel (the tail message)
    UpdateChannel(ChannelAccount),

    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    SendMessage(SendMessage),
    // Add message to message builder
    //BuildMessagePart(String),

    // Submit message from BuildMessage invocations
    //SubmitMessage,
    CreatePost(CreatePost),

    CreatePostContent(CreatePostContent),

    // "Like" the post with an amount
    StakePost(StakePost),
}
