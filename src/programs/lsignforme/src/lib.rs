#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod names;
pub mod processor;
pub mod state;

solana_program::declare_id!("HGXKsXGRx9qSyrNS6YAEd5FqjGMXLf41syf9jhNkbvwn");
use solana_program::pubkey::Pubkey;

const SIGN_FOR_ME_RECORD_SEED: &[u8] = b"s4m";
pub fn get_sign_for_me_program_address(
    program_id: &Pubkey,
    owner: &Pubkey,
    signer: &Pubkey,
    scope: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SIGN_FOR_ME_RECORD_SEED,
            owner.as_ref(),
            signer.as_ref(),
            scope.as_ref(),
        ],
        program_id,
    )
}

pub fn get_sign_for_me_program_address_seeds<'a>(
    owner: &'a Pubkey,
    signer: &'a Pubkey,
    scope: &'a Pubkey,
    bump_seed: &'a [u8; 1],
) -> [&'a [u8]; 5] {
    [
        SIGN_FOR_ME_RECORD_SEED,
        owner.as_ref(),
        signer.as_ref(),
        scope.as_ref(),
        bump_seed,
    ]
}
