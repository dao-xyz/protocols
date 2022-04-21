#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod names;
pub mod processor;
pub mod state;

solana_program::declare_id!("FHnDtK9D2MDSKWvyD4eLk7CZfUa9FP4zH77TKiVAeDXK");
use shared::seeds::generate_seeds_from_string;
use solana_program::pubkey::Pubkey;
const TAG_SEED: &[u8] = b"tag";

pub fn get_tag_program_address(program_id: &Pubkey, tag: &str) -> (Pubkey, u8) {
    let seeds = get_tag_program_address_seeds(tag);
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Pubkey::find_program_address(seed_slice, program_id)
}

pub fn get_tag_program_address_seeds(tag: &str) -> Vec<Vec<u8>> {
    let mut seeds = generate_seeds_from_string(tag).unwrap();
    seeds.insert(0, TAG_SEED.to_vec());
    seeds
}

pub fn get_tag_record_program_address(
    program_id: &Pubkey,
    factory: &Pubkey,
    owner: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"record", factory.as_ref(), owner.as_ref()], program_id)
}

pub fn get_tag_record_program_address_seeds<'a>(
    factory: &'a Pubkey,
    owner: &'a Pubkey,
    bump_seed: &'a [u8; 1],
) -> [&'a [u8]; 4] {
    [b"record", factory.as_ref(), owner.as_ref(), bump_seed]
}

pub fn get_tag_record_factory_program_address(
    program_id: &Pubkey,
    tag: &Pubkey,
    authority: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"record_factory", tag.as_ref(), authority.as_ref()],
        program_id,
    )
}

pub fn get_tag_record_factory_program_address_seeds<'a>(
    tag: &'a Pubkey,
    authority: &'a Pubkey,
    bump_seed: &'a [u8; 1],
) -> [&'a [u8]; 4] {
    [
        b"record_factory",
        tag.as_ref(),
        authority.as_ref(),
        bump_seed,
    ]
}
