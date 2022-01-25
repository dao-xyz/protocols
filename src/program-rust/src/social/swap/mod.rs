use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::{Pubkey, PubkeyError},
    rent::Rent,
    system_instruction::{self, create_account},
    sysvar::Sysvar,
};
pub mod identity;
pub mod longshort;
//pub mod offset;

use crate::tokens::spl_utils::{
    create_mint_escrow_program_address_seeds, create_program_account_mint_account_with_seed,
    create_program_token_account, find_mint_escrow_program_address, spl_mint_to,
};

use super::create_post_mint_authority_program_address_seeds;

pub fn find_escrow_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    find_mint_escrow_program_address(program_id, post)
}

pub fn create_escrow_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_escrow_program_address_seeds(post, bump_seed)
}
