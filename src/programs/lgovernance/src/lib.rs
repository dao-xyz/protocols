pub mod accounts;
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod pack;
pub mod processor;
pub mod shared;
pub mod state;
pub mod tokens;
solana_program::declare_id!("GhhdZ7v99edo9v6XmitqEoKT5jev1mpCpVWim6bgKsh1");

use solana_program::pubkey::Pubkey;

const PROGRAM_AUTHORITY_SEED: &[u8] = b"p_authority";

const DELEGATEE_SEED: &[u8] = b"delegatee";

/// Find treasury account address

pub fn find_treasury_token_account_address(
    program_id: &Pubkey,
    channel: &Pubkey,
    spl_token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &channel.to_bytes(),
            &token_program_id.to_bytes(),
            &spl_token_mint_address.to_bytes(),
        ],
        program_id,
    )
}

pub fn create_treasury_token_account_address_seeds<'a>(
    channel: &'a Pubkey,
    spl_token_mint_address: &'a Pubkey,
    token_program_id: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        channel.as_ref(),
        token_program_id.as_ref(),
        spl_token_mint_address.as_ref(),
        bump_seed,
    ]
}
