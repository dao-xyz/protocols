use solana_program::pubkey::{Pubkey, PubkeyError};

use crate::{
    address::generate_seeds_from_string,
    tokens::spl_utils::{
        create_mint_authority_program_address_seeds, create_mint_escrow_program_address_seeds,
        create_mint_program_address_seeds, find_mint_authority_program_address,
        find_mint_escrow_program_address, find_mint_program_address,
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

///  Seed for create token accounts for associated mint with the post
const POST_TOKEN: &[u8] = b"post_token";

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

/// Find user post channel address
pub fn find_post_program_address(
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

/// create user post channel address
/* pub fn create_post_program_address<'a>(
    user: &'a Pubkey,
    channel: &'a Pubkey,
    timestamp_le_bytes: &'a [u8],
    bump_seed: &'a [u8],
) -> &'a [[u8]; 4] {
    &[
        &user.to_bytes(),
        &channel.to_bytes(),
        timestamp_le_bytes,
        bump_seed,
    ]
} */

/// Find post content
pub fn find_post_content_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&post.to_bytes()], program_id)
}

/// create post content address
pub fn create_post_content_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
    bump_seed: &[u8],
) -> Result<Pubkey, PubkeyError> {
    Pubkey::create_program_address(&[&post.to_bytes(), bump_seed], program_id)
}

/// Find address for the token mint for the post account
pub fn find_post_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    find_mint_program_address(program_id, post)
}

/// Create post mint program address
pub fn create_post_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_program_address_seeds(post, bump_seed)
}

/// Find address for the token mint for the post account
pub fn find_post_mint_authority_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
) -> (Pubkey, u8) {
    find_mint_authority_program_address(program_id, post)
}

/// Create post mint authority program address
pub fn create_post_mint_authority_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_authority_program_address_seeds(mint, bump_seed)
}

/// Find escrow address for mint
pub fn find_post_escrow_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    find_mint_escrow_program_address(program_id, post)
}

/// Create post mint escrow program address
pub fn create_post_mint_escrow_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_escrow_program_address_seeds(mint, bump_seed)
}

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
