use solana_program::pubkey::{Pubkey, PubkeyError};

use super::generate_seeds_from_string;

pub mod instruction;
pub mod processor;
pub mod state;

/// Find channel address from name
pub fn find_channel_program_address(
    program_id: &Pubkey,
    channel_name: &str,
) -> Result<(Pubkey, u8), PubkeyError> {
    let seeds = create_channel_account_program_address_seeds(channel_name)?;
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Ok(Pubkey::find_program_address(seed_slice, program_id))
}

/// Create post mint program address
pub fn create_channel_account_program_address_seeds(
    channel_name: &str,
) -> Result<Vec<Vec<u8>>, PubkeyError> {
    /* let mut seeds = generate_seeds_from_string(channel_name).unwrap();
    seeds.push(CHANNEL.to_vec());
    seeds.reverse();
    seeds */
    generate_seeds_from_string(channel_name)
}
