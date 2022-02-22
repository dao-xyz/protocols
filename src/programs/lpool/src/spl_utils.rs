//! SPL Utilities

use solana_program::{program::{invoke, invoke_signed}, account_info::AccountInfo, program_error::ProgramError, msg, system_instruction, pubkey::Pubkey, program_pack::Pack, rent::Rent, entrypoint::ProgramResult, sysvar::Sysvar};
use spl_token::{instruction::initialize_mint, state::Mint};


/// Create a mint owned by a program
#[allow(clippy::too_many_arguments)]
pub fn create_program_account_mint_account_with_seed<'a>(
    mint_info: &AccountInfo<'a>,
    mint_account_seeds: &[&[u8]],
    mint_authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let decimals = spl_token::native_mint::DECIMALS; // for now
    let address = Pubkey::create_program_address(mint_account_seeds, program_id).unwrap();
    if mint_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            mint_info.key,
            mint_rent,
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            mint_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[mint_account_seeds],
    )?;

    invoke(
        &initialize_mint(
            &spl_token::id(),
            mint_info.key,
            mint_authority_info.key,
            None,
            decimals,
        )?,
        &[mint_info.clone(), rent_info.clone()],
    )?;
    Ok(())
}


/// Issue a spl_token `Transfer` instruction.
#[allow(clippy::too_many_arguments)]
pub fn token_transfer<'a>(
    token_program: AccountInfo<'a>,
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::transfer(
        token_program.key,
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;
    invoke(&ix, &[source, destination, authority, token_program])
}
