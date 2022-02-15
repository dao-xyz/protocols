use arrayref::array_ref;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_token::{
    instruction::{initialize_account, initialize_mint, mint_to},
    state::Mint,
};

use crate::pack::check_data_len;

pub const MINT_SEED: &[u8] = b"mint";
pub const UTILITY_MINT: &[u8] = b"utility";

pub const MINT_AUTHORTY_SEED: &[u8] = b"authority";
pub const ESCROW_ACCOUNT_SEED: &[u8] = b"escrow";

/// Find utility mint token address (unique/fixed for program)
pub fn find_utility_mint_program_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, UTILITY_MINT], program_id)
}

/// Create mint address (unique/fixed for program)
pub fn create_utility_mint_program_address_seeds(bump_seed: &[u8]) -> [&[u8]; 3] {
    [MINT_SEED, UTILITY_MINT, bump_seed]
}

/// Find mint address
pub fn find_mint_program_address(program_id: &Pubkey, some_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, &some_account.to_bytes()], program_id)
}

/// Create mint address
pub fn create_mint_program_address_seeds<'a>(
    account_seed: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [MINT_SEED, account_seed.as_ref(), bump_seed]
}

/// Generate mint authority address
pub fn find_mint_authority_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_AUTHORTY_SEED, &mint.to_bytes()], program_id)
}

/// Create mint authority address
pub fn create_mint_authority_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [MINT_AUTHORTY_SEED, mint.as_ref(), bump_seed]
}

/// Generate mint escrow address
pub fn find_mint_escrow_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ESCROW_ACCOUNT_SEED, &mint.to_bytes()], program_id)
}

/// Create mint authority address
pub fn create_mint_escrow_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [ESCROW_ACCOUNT_SEED, mint.as_ref(), bump_seed]
}

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

#[allow(clippy::too_many_arguments)]
pub fn create_program_token_account<'a>(
    account_info: &AccountInfo<'a>,
    account_seeds: &[&[u8]],
    mint_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let address = Pubkey::create_program_address(account_seeds, program_id).unwrap();
    if account_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            account_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            account_info.key,
            rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            account_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[account_seeds], // missing things here, we need the full seed for the mint accoutn
    )?;

    invoke(
        &initialize_account(
            token_program_info.key,
            account_info.key,
            mint_info.key,
            authority_info.key, //freeze_authority_pubkey.as_ref(),
        )?,
        &[
            account_info.clone(),
            mint_info.clone(),
            authority_info.clone(),
            rent_info.clone(),
        ],
    )?;
    Ok(())
}

pub fn spl_mint_to<'a>(
    mint_to_account: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    mint_authority_seeds: &[&[u8]],
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let mint_authority_address =
        Pubkey::create_program_address(mint_authority_seeds, program_id).unwrap();
    if mint_authority_info.key != &mint_authority_address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_authority_info.key,
            mint_authority_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let ix = mint_to(
        &spl_token::id(),
        mint_info.key,
        mint_to_account.key,
        &mint_authority_address,
        &[],
        amount,
    )?;
    invoke_signed(
        &ix,
        &[
            mint_info.clone(),
            mint_to_account.clone(),
            mint_authority_info.clone(),
        ],
        &[mint_authority_seeds],
    )?;

    Ok(())
}

pub fn transfer_to<'a>(
    payer_info: &AccountInfo<'a>,
    to_account_info: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    // take from payer
    invoke(
        &system_instruction::transfer(
            payer_info.key, // mby this should be program id
            to_account_info.key,
            amount,
        ),
        &[payer_info.clone(), to_account_info.clone()],
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

/// Issue a spl_token `Transfer` instruction signed.
#[allow(clippy::too_many_arguments)]
pub fn token_transfer_signed<'a>(
    token_program: AccountInfo<'a>,
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    seeds: &[&[u8]],
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
    invoke_signed(
        &ix,
        &[source, destination, authority, token_program],
        &[seeds],
    )
}

/// Returns Token Mint supply.
/// Extrats supply field without unpacking entire struct.
pub fn get_token_supply(token_mint: &AccountInfo) -> Result<u64, ProgramError> {
    let data = token_mint.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Mint::get_packed_len())?;
    let supply = array_ref![data, 36, 8];

    Ok(u64::from_le_bytes(*supply))
}
