use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    instruction::CHAT_INSTRUCTION_INDEX,
    shared::io_utils::try_to_vec_prepend,
    social::accounts::{ChannelAccount, Message, UserAccount},
    tokens::spl_utils::{
        find_mint_authority_program_address, find_mint_escrow_program_address,
        find_utility_mint_program_address,
    },
};

use super::{
    accounts::{AMMCurve, Content, Description, MarketMaker},
    find_channel_program_address, find_post_downvote_mint_program_address,
    find_post_escrow_program_address, find_post_mint_authority_program_address,
    find_post_program_address, find_post_upvote_mint_program_address,
    find_user_account_program_address, Vote,
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
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub content: Content,
    pub market_maker: MarketMaker,
    pub post_bump_seed: u8,
    pub mint_upvote_bump_seed: u8,
    pub mint_downvote_bump_seed: u8,
    pub escrow_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
}

/* #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePostContent {
    pub message: Message,
    pub bump_seed: u8,
} */

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct VotePost {
    pub post: Pubkey,
    pub stake: u64,
    pub mint_authority_bump_seed: u8,
    pub mint_escrow_bump_seed: u8,
    pub vote: Vote,
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
    //CreatePostContent(CreatePostContent),

    // "Like" or "Dislike" the post with an amount of Solvei tokens
    VotePost(VotePost),
    UnvotePost(VotePost),
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
            AccountMeta::new(*payer, true),
            AccountMeta::new(user_address_pda, false),
            AccountMeta::new(system_program::id(), false),
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
            AccountMeta::new(*payer, true),
            AccountMeta::new(*user, false),
            AccountMeta::new(channel, false),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

/// Creates a create post transction
pub fn create_post_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    user: &Pubkey,
    channel: &Pubkey,
    timestamp: u64,
    content: &Content,
    market_maker: &MarketMaker,
) -> Instruction {
    let (post_address, post_bump_seed) = find_post_program_address(program_id, &content.hash);
    let (mint_upvote_address, mint_upvote_bump_seed) =
        find_post_upvote_mint_program_address(program_id, &post_address);

    let (mint_downvote_address, mint_downvote_bump_seed) =
        find_post_downvote_mint_program_address(program_id, &post_address);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, &post_address);

    let (escrow_address, escrow_bump_seed) =
        find_post_escrow_program_address(program_id, &post_address);

    let (utility_mint_address, _) = find_utility_mint_program_address(program_id);
    /*   let (user_post_token_account, user_post_token_account_bump_seed) =
    find_user_post_token_program_address(program_id, &post_address, user); */
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*user, false),
        AccountMeta::new(post_address, false),
        AccountMeta::new(mint_upvote_address, false),
        AccountMeta::new(mint_downvote_address, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(escrow_address, false),
        AccountMeta::new(utility_mint_address, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    // For offset SwapCurve we need aditional accounts
    if let MarketMaker::AMM(AMMCurve::Offset(_)) = market_maker {
        /*

        let swap_account_info = next_account_info(accounts_iter)?;
        let swap_authority_info = next_account_info(accounts_iter)?;
        let token_a_account = next_account_info(accounts_iter)?;
        let token_b_account = next_account_info(accounts_iter)?;
        let swap_pool_mint = next_account_info(accounts_iter)?;
        let swap_pool_token_account = next_account_info(accounts_iter)?;
        let swap_initial_token_account = next_account_info(accounts_iter)?;


        */

        let (swap_address, _) =
            Pubkey::find_program_address(&[b"SWAP", post_address.as_ref()], &program_id);
        let (swap_authorty_address, bump_seed) =
            Pubkey::find_program_address(&[&swap_address.to_bytes()[..]], &program_id);
        let (token_a_adress, bump_seed) =
            Pubkey::find_program_address(&[b"MINT", &swap_address.to_bytes()[..]], &program_id);
        let (swap_mint_address, bump_seed) =
            Pubkey::find_program_address(&[b"MINT", &swap_address.to_bytes()[..]], &program_id);
        let (swap_authorty_address, bump_seed) =
            Pubkey::find_program_address(&[&swap_address.to_bytes()[..]], &program_id);
    }

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreatePost(CreatePost {
            creator: *user,
            channel: *channel,
            mint_upvote_bump_seed,
            mint_downvote_bump_seed,
            mint_authority_bump_seed,
            market_maker: market_maker.clone(),
            timestamp,
            content: content.clone(),
            post_bump_seed,
            escrow_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

// "Stake" solvei tokens
pub fn create_post_vote_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_address, _) = match vote {
        Vote::UP => find_post_upvote_mint_program_address(program_id, post),
        Vote::DOWN => find_post_downvote_mint_program_address(program_id, post),
    };
    let payer_utility_token_address =
        get_associated_token_address(payer, &find_utility_mint_program_address(program_id).0);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);
    let (escrow_address, mint_escrow_bump_seed) =
        find_post_escrow_program_address(program_id, post);
    let associated_token_address = get_associated_token_address(payer, &mint_address);

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::VotePost(VotePost {
            mint_authority_bump_seed,
            stake,
            post: *post,
            mint_escrow_bump_seed,
            vote,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(payer_utility_token_address, false),
            AccountMeta::new(*post, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(mint_authority_address, false),
            AccountMeta::new(associated_token_address, false),
            AccountMeta::new(escrow_address, false),
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
    }
}

// "Unstake" solvei tokens
pub fn create_post_unvote_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_address, _) = match vote {
        Vote::UP => find_post_upvote_mint_program_address(program_id, post),
        Vote::DOWN => find_post_downvote_mint_program_address(program_id, post),
    };
    let payer_utility_token_address =
        get_associated_token_address(payer, &find_utility_mint_program_address(program_id).0);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);
    let (escrow_address, mint_escrow_bump_seed) =
        find_post_escrow_program_address(program_id, post);
    let associated_token_address = get_associated_token_address(payer, &mint_address);

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::UnvotePost(VotePost {
            mint_authority_bump_seed,
            stake,
            post: *post,
            mint_escrow_bump_seed,
            vote,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(payer_utility_token_address, false),
            AccountMeta::new(*post, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new(mint_authority_address, false),
            AccountMeta::new(associated_token_address, false),
            AccountMeta::new(escrow_address, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    }
}
