use std::{io::Error, iter::FromIterator};
use solana_program::pubkey::{MAX_SEEDS, MAX_SEED_LEN, Pubkey, PubkeyError};
pub use solana_program;

solana_program::declare_id!("c39Hxxzh7Sh3GgkZM1QzMDyT5Q5cjK5397sbqeBrB1Q");
 


/**
 * Generete seed slices from string 
 * in correct length (max length 32 bytes)
 */
pub fn generate_seeds_from_string<'a> (str: &'a str) -> Result<Vec<Vec<u8>>, PubkeyError>
{
    let seeds =  str.chars().collect::<Vec<char>>()
        .chunks(MAX_SEED_LEN)
        .map(|char| {
        return String::from_iter(char).as_bytes().iter().map(|x| *x).collect::<Vec<_>>();
    }).collect::<Vec<_>>();
    if seeds.len() > MAX_SEEDS
    {
        return Err(PubkeyError::MaxSeedLengthExceeded)
    }
    Ok(seeds)
}   


mod test {
    use solana_program::pubkey::{MAX_SEEDS, MAX_SEED_LEN};

    use crate::address::{generate_seeds_from_string};


    #[test]
    fn test_generate_seeds_from_string(){
        let seed_string = (0..MAX_SEED_LEN*MAX_SEEDS).map(|_| "X").collect::<String>();
        let generated_seeds = generate_seeds_from_string(seed_string.as_str()).unwrap();
        assert_eq!(generated_seeds.len(), MAX_SEEDS);
        generated_seeds.iter().for_each(|seed| assert_eq!(seed.len(), MAX_SEED_LEN));
    }

    #[test]
    fn test_generate_seeds_from_string_to_long() {
        let seed_string = (0..MAX_SEED_LEN*MAX_SEEDS + 1).map(|_| "X").collect::<String>();
        let generated_seeds = generate_seeds_from_string(seed_string.as_str());
        assert!(generated_seeds.is_err());
    }
}