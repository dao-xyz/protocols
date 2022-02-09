use std::iter::FromIterator;

use borsh::BorshSerialize;
use solana_program::account_info::AccountInfo;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub mod channel;
pub mod instruction;
pub mod post;
pub mod processor;
pub mod state;
pub mod user;

use solana_program::pubkey::{PubkeyError, MAX_SEEDS, MAX_SEED_LEN};
use solana_program::rent::Rent;
use solana_program::{msg, system_instruction};

pub trait MaxSize {
    /// Returns max account size or None if max size is not known and actual instance size should be used
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

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

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_signed_verify<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let account_address_pda = Pubkey::create_program_address(seeds, program_id)?;
    if account_info.key != &account_address_pda {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            account_address_pda,
            account_info.key
        );
        return Err(ProgramError::InvalidSeeds);
    }
    create_and_serialize_account_signed_from_pda(
        payer_info,
        account_info,
        account_data,
        seeds,
        program_id,
        system_info,
        rent,
    )
}

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
fn create_and_serialize_account_signed_from_pda<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let (serialized_data, account_size) = if let Some(max_size) = account_data.get_max_size() {
        (None, max_size)
    } else {
        let serialized_data = account_data.try_to_vec()?;
        let account_size = serialized_data.len();
        (Some(serialized_data), account_size)
    };

    let create_account_instruction = system_instruction::create_account(
        payer_info.key,
        account_info.key,
        rent.minimum_balance(account_size),
        account_size as u64,
        program_id,
    );
    invoke_signed(
        &create_account_instruction,
        &[
            payer_info.clone(),
            account_info.clone(),
            system_info.clone(),
        ],
        &[seeds],
    )?;

    if let Some(serialized_data) = serialized_data {
        account_info
            .data
            .borrow_mut()
            .copy_from_slice(&serialized_data);
    } else {
        account_data.serialize(&mut *account_info.data.borrow_mut())?;
        // account_info.data will be empty after this even though some value has been serialized
    }

    Ok(())
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
