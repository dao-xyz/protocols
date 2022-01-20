use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::{Pubkey, PubkeyError},
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction::initialize_mint, state::Mint};

use crate::{
    address::generate_seeds_from_string,
    tokens::spl_utils::{
        create_mint_authority_program_address_seeds, create_mint_escrow_program_address_seeds,
        create_mint_program_address_seeds, find_mint_authority_program_address,
        find_mint_escrow_program_address, find_mint_program_address, MINT_SEED,
    },
};

pub mod accounts;
pub mod instruction;
pub mod processor;
pub mod rates;

/// Seed for user accounts
const USER: &[u8] = b"user";

/// Seed for channel
const CHANNEL: &[u8] = b"channel";

/// Seed for UPVOTE
const UPVOTE: &[u8] = b"up";

/// Seed for downvote
const DOWNVOTE: &[u8] = b"down";

/// Seed for SWAP account
const SWAP: &[u8] = b"swap";

/// Seed for swap utitlity token account
const SWAP_UTILITY: &[u8] = b"utility";

/// Seed for swap vote (up|down) token account
const SWAP_VOTE: &[u8] = b"vote";

/// Seed for swap fee account
const SWAP_FEE: &[u8] = b"swap_fee";

/// Seed for swap destination token account
const SWAP_DESTINATION: &[u8] = b"swap_fee";

/// Seed for swap destination token account
const SWAP_MINT: &[u8] = b"swap_mint";

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Vote {
    UP,
    DOWN,
}

/// Find user account program owned address from username
pub fn find_user_account_program_address(program_id: &Pubkey, username: &str) -> (Pubkey, u8) {
    let seeds = create_user_account_program_address_seeds(username);
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Pubkey::find_program_address(seed_slice, program_id)
}

/// Create post mint program address
pub fn create_user_account_program_address_seeds(username: &str) -> Vec<Vec<u8>> {
    let mut seeds = generate_seeds_from_string(username).unwrap();
    seeds.push(USER.to_vec());
    seeds.reverse();
    seeds
}

/// Find channel address from name
pub fn find_channel_program_address(program_id: &Pubkey, channel_name: &str) -> (Pubkey, u8) {
    let seeds = create_channel_account_program_address_seeds(channel_name);
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Pubkey::find_program_address(seed_slice, program_id)
}

/// Create post mint program address
pub fn create_channel_account_program_address_seeds(channel_name: &str) -> Vec<Vec<u8>> {
    let mut seeds = generate_seeds_from_string(channel_name).unwrap();
    seeds.push(CHANNEL.to_vec());
    seeds.reverse();
    seeds
}

pub fn create_post_mint_program_account<'a>(
    post: &Pubkey,
    vote: Vote,

    mint_info: &AccountInfo<'a>,
    mint_bump_seed: u8,
    mint_authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let decimals = 9; // for now
    let mint_bump_seed = &[mint_bump_seed];
    let mint_account_seeds = match vote {
        Vote::UP => create_post_upvote_mint_program_address_seeds(post, mint_bump_seed),
        Vote::DOWN => create_post_downvote_mint_program_address_seeds(post, mint_bump_seed),
    };

    let address = Pubkey::create_program_address(&mint_account_seeds, program_id).unwrap();
    if mint_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            mint_info.key,
            mint_rent,
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            mint_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[&mint_account_seeds],
    )?;

    invoke(
        &initialize_mint(
            &spl_token::id(),
            mint_info.key,
            mint_authority_info.key,
            None,
            decimals,
        )?,
        &[mint_info.clone(), rent_info.clone()],
    )?;
    Ok(())
}

/// Find user post channel address
/* pub fn find_post_program_address(
    program_id: &Pubkey,
    user: &Pubkey,
    channel: &Pubkey,
    timestamp: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &user.to_bytes(),
            &channel.to_bytes(),
            &timestamp.to_le_bytes(),
        ],
        program_id,
    )
}
 */

pub fn find_post_program_address(program_id: &Pubkey, hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[hash], program_id)
}

/// Find address for the token upvote mint for the post account
pub fn find_post_upvote_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, UPVOTE, &post.to_bytes()], program_id)
}

/// Create post mint upvote program address
pub fn create_post_upvote_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [MINT_SEED, UPVOTE, post.as_ref(), bump_seed]
}

/// Find address for the token downvote mint for the post account
pub fn find_post_downvote_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, DOWNVOTE, &post.to_bytes()], program_id)
}

/// Create post mint downvote program address
pub fn create_post_downvote_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [MINT_SEED, DOWNVOTE, post.as_ref(), bump_seed]
}

/// Find address for the token mint authority for the post account
pub fn find_post_mint_authority_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
) -> (Pubkey, u8) {
    find_mint_authority_program_address(program_id, post)
}

/// Create post mint authority program address
pub fn create_post_mint_authority_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_authority_program_address_seeds(post, bump_seed)
}

/// Find escrow address for mint
pub fn find_post_escrow_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    find_mint_escrow_program_address(program_id, post)
}

/// Create post mint escrow program address
pub fn create_post_mint_escrow_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_escrow_program_address_seeds(post, bump_seed)
}

/*
/// Find address for the token mint for the post account
pub fn find_user_post_token_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
    user: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&POST_TOKEN, &post.to_bytes(), &user.to_bytes()],
        program_id,
    )
}

/// Create post mint escrow program address
pub fn create_user_post_token_program_address_seeds<'a>(
    post: &'a Pubkey,
    user: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [&POST_TOKEN, post.as_ref(), user.as_ref(), bump_seed]
}
 */
