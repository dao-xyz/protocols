pub mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod shared;
pub mod state;
pub mod tokens;

solana_program::declare_id!("758spmW8vgAsBX6LdpDVemeEUzwRtoCbJYbPbyf7iFtn");
pub mod lpost {
    solana_program::declare_id!("758spmW8vgAsBX6LdpDVemeEUzwRtoCbJYbPbyf7iFtn");
}
use luser::generate_seeds_from_string;

use solana_program::pubkey::{Pubkey, PubkeyError};

/// Findchannel address from name
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
    generate_seeds_from_string(channel_name)
}
