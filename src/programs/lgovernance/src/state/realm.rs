use solana_program::pubkey::Pubkey;
/*
use crate::accounts::AccountType;

// The root of an org
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Realm {
    acccount_type: AccountType,
}

pub fn create_realm_program_address_seeds(name: &str) -> Result<Vec<Vec<u8>>, PubkeyError> {
    let mut seeds = generate_seeds_from_string(name)?;
    seeds.push(b"realm".to_vec());
    seeds.reverse();
    Ok(seeds)
}
pub fn find_realm_program_address(
    name: &str,
    program_id: &Pubkey,
) -> Result<(Pubkey, u8), PubkeyError> {
    let seeds = create_realm_program_address_seeds(name)?;
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Ok(Pubkey::find_program_address(seed_slice, program_id))
}
 */
const REALM_SEED: &[u8] = b"realm";
const REALM_AUTHORITY_SEED: &[u8] = b"realm_authority";

pub fn get_realm_mint_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [REALM_SEED, mint.as_ref(), bump_seed]
}
pub fn get_realm_mint_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REALM_SEED, mint.as_ref()], program_id)
}

pub fn get_realm_mint_authority_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [REALM_SEED, mint.as_ref(), bump_seed]
}
pub fn get_realm_mint_authority_program_address(
    program_id: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REALM_AUTHORITY_SEED, mint.as_ref()], program_id)
}
