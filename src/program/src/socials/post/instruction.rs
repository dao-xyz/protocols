use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    socials::instruction::SocialInstruction, tokens::spl_utils::find_utility_mint_program_address,
};

use super::{
    find_escrow_program_address, find_post_downvote_mint_program_address,
    find_post_mint_authority_program_address, find_post_program_address,
    find_post_upvote_mint_program_address,
    state::{Content, PostAccount},
    Vote,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePost {
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub content: Content,
    pub post_bump_seed: u8,
    pub escrow_bump_seed: u8,
    pub mint_upvote_bump_seed: u8,
    pub mint_downvote_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostVote {
    pub post: Pubkey,
    pub stake: u64,
    pub vote: Vote,
    pub mint_authority_bump_seed: u8,
    pub escrow_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostInstruction {
    // Create channel
    CreatePost(CreatePost),
    Vote(PostVote),
    Unvote(PostVote),
}

pub fn create_post_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    user: &Pubkey,
    channel: &Pubkey,
    content: &Content,
) -> Instruction {
    let (post_address, post_bump_seed) = find_post_program_address(program_id, &content.hash);
    let (mint_upvote_address, mint_upvote_bump_seed) =
        find_post_upvote_mint_program_address(program_id, &post_address);

    let (mint_downvote_address, mint_downvote_bump_seed) =
        find_post_downvote_mint_program_address(program_id, &post_address);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, &post_address);

    let (utility_mint_address, _) = find_utility_mint_program_address(program_id);
    /*   let (user_post_token_account, user_post_token_account_bump_seed) =
    find_user_post_token_program_address(program_id, &post_address, user); */
    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*user, false),
        AccountMeta::new(post_address, false),
        AccountMeta::new(mint_upvote_address, false),
        AccountMeta::new(mint_downvote_address, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(utility_mint_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    // For offset SwapCurve we need aditional accounts
    let (escrow_address, escrow_bump_seed) = find_escrow_program_address(program_id, &post_address);
    accounts.push(AccountMeta::new(escrow_address, false));

    Instruction {
        program_id: *program_id,
        data: SocialInstruction::PostInstruction(PostInstruction::CreatePost(CreatePost {
            creator: *user,
            channel: *channel,
            mint_upvote_bump_seed,
            mint_downvote_bump_seed,
            mint_authority_bump_seed,
            escrow_bump_seed,
            content: content.clone(),
            post_bump_seed,
        }))
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
    _post_account: &PostAccount,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_upvote_address, _) = find_post_upvote_mint_program_address(program_id, post);
    let (mint_downvote_address, _) = find_post_downvote_mint_program_address(program_id, post);

    let payer_utility_token_address =
        get_associated_token_address(payer, &find_utility_mint_program_address(program_id).0);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);

    let associated_token_address = match vote {
        Vote::UP => get_associated_token_address(payer, &mint_upvote_address),
        Vote::DOWN => get_associated_token_address(payer, &mint_downvote_address),
    };

    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(payer_utility_token_address, false),
        AccountMeta::new(*post, false),
        AccountMeta::new(mint_upvote_address, false),
        AccountMeta::new(mint_downvote_address, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(associated_token_address, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    let (escrow_address, escrow_bump_seed) = find_escrow_program_address(program_id, post);
    accounts.push(AccountMeta::new(escrow_address, false));

    Instruction {
        program_id: *program_id,
        data: SocialInstruction::PostInstruction(PostInstruction::Vote(PostVote {
            mint_authority_bump_seed,
            stake,
            post: *post,
            escrow_bump_seed,
            vote,
        }))
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

// "Unstake" solvei tokens
pub fn create_post_unvote_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    _post_account: &PostAccount,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_upvote_address, _) = find_post_upvote_mint_program_address(program_id, post);
    let (mint_downvote_address, _) = find_post_downvote_mint_program_address(program_id, post);

    let payer_utility_token_address =
        get_associated_token_address(payer, &find_utility_mint_program_address(program_id).0);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);

    let associated_token_address = match vote {
        Vote::UP => get_associated_token_address(payer, &mint_upvote_address),
        Vote::DOWN => get_associated_token_address(payer, &mint_downvote_address),
    };

    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(payer_utility_token_address, false),
        AccountMeta::new(*post, false),
        AccountMeta::new(mint_upvote_address, false),
        AccountMeta::new(mint_downvote_address, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(associated_token_address, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let (escrow_address, escrow_bump_seed) = find_escrow_program_address(program_id, post);
    accounts.push(AccountMeta::new(escrow_address, false));
    Instruction {
        program_id: *program_id,
        data: SocialInstruction::PostInstruction(PostInstruction::Unvote(PostVote {
            mint_authority_bump_seed,
            stake,
            post: *post,
            escrow_bump_seed,
            vote,
        }))
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}
