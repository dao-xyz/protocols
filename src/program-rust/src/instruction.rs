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
pub struct InitializeToken {
    pub mint_bump_seed: u8,
    pub escrow_bump_seed: u8,
    pub multisig_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct InitializeStakePool {
    pub stake_pool_bump_seed: u8,
    // pub manager_bump_seed: u8,
    pub manager_fee_account_bump_seed: u8,
    pub pool_mint_bump_seed: u8,
    pub reserve_stake_bump_seed: u8,
    pub validator_list_bump_seed: u8,
    pub stake_pool_packed_len: u64,
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

    // Initialize utility token (solvei token)
    InitializeToken(InitializeToken),

    SetupStakePool(InitializeStakePool),

    InitializeStakePool(InitializeStakePool),

    StakePoolInstruction(StakePoolInstruction),
}
