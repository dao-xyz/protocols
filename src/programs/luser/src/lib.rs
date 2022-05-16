#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod shared;
pub mod state;

solana_program::declare_id!("FeHc2yfKLCSxdwyvKaY8rZsA4da3kyJNpjJtQe8ioh87");
use solana_program::pubkey::{Pubkey, PubkeyError, MAX_SEEDS, MAX_SEED_LEN};
use std::iter::FromIterator;

/// Find user account program owned address from username
pub fn find_user_account_program_address(program_id: &Pubkey, username: &str) -> (Pubkey, u8) {
    let seeds = create_user_account_program_address_seeds(username);
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Pubkey::find_program_address(seed_slice, program_id)
}

/// Create post mint program address
pub fn create_user_account_program_address_seeds(username: &str) -> Vec<Vec<u8>> {
    generate_seeds_from_string(username).unwrap()
}

/**
 * Generete seed slices from string
 * in correct length (max length 32 bytes)
 * Will perform lowercase before generating seed
 */
pub fn generate_seeds_from_string(str: &str) -> Result<Vec<Vec<u8>>, PubkeyError> {
    let seeds = str
        .chars()
        .map(|c| c.to_lowercase())
        .flatten()
        .collect::<Vec<char>>()
        .chunks(MAX_SEED_LEN)
        .map(|char| {
            return String::from_iter(char)
                .as_bytes()
                .iter()
                .copied()
                .collect::<Vec<_>>();
        })
        .collect::<Vec<_>>();
    if seeds.len() > MAX_SEEDS {
        return Err(PubkeyError::MaxSeedLengthExceeded);
    }
    Ok(seeds)
}

#[cfg(test)]
mod test {

    use super::*;
    use solana_program::pubkey::{MAX_SEEDS, MAX_SEED_LEN};

    #[test]
    fn test_generate_seeds_from_string() {
        let seed_string = (0..MAX_SEED_LEN * MAX_SEEDS)
            .map(|_| "X")
            .collect::<String>();
        let generated_seeds = generate_seeds_from_string(seed_string.as_str()).unwrap();
        assert_eq!(generated_seeds.len(), MAX_SEEDS);
        generated_seeds
            .iter()
            .for_each(|seed| assert_eq!(seed.len(), MAX_SEED_LEN));
    }

    #[test]
    fn test_generate_seeds_from_string_to_long() {
        let seed_string = (0..MAX_SEED_LEN * MAX_SEEDS + 1)
            .map(|_| "X")
            .collect::<String>();
        let generated_seeds = generate_seeds_from_string(seed_string.as_str());
        assert!(generated_seeds.is_err());
    }
}
