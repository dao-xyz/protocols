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

pub const MINT_SEED: &[u8] = b"mint";
pub const UTILITY_MINT: &[u8] = b"utility";

pub const AUTHORTY_SEED: &[u8] = b"authority";
pub const ESCROW_ACCOUNT_SEED: &[u8] = b"escrow";

/// Find utility mint token address (unique/fixed for program)
pub fn find_platform_mint_program_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, UTILITY_MINT], program_id)
}

/// Create mint address (unique/fixed for program)
pub fn create_platform_mint_program_address_seeds(bump_seed: &[u8]) -> [&[u8]; 3] {
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
pub fn find_authority_program_address(program_id: &Pubkey, key: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[AUTHORTY_SEED, &key.to_bytes()], program_id)
}

/// Create mint authority address
pub fn create_authority_program_address_seeds<'a>(
    key: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [AUTHORTY_SEED, key.as_ref(), bump_seed]
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

/// Creates and initializes SPL token account with PDA using the provided PDA seeds with bump included
#[allow(clippy::too_many_arguments)]
pub fn create_spl_token_account_signed_with_bump<'a>(
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

/// Creates and initializes SPL token account with PDA using the provided PDA seeds
#[allow(clippy::too_many_arguments)]
pub fn create_spl_token_account_signed<'a>(
    payer_info: &AccountInfo<'a>,
    token_account_info: &AccountInfo<'a>,
    token_account_address_seeds: &[&[u8]],
    token_mint_info: &AccountInfo<'a>,
    token_account_owner_info: &AccountInfo<'a>,
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    spl_token_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let create_account_instruction = system_instruction::create_account(
        payer_info.key,
        token_account_info.key,
        1.max(rent.minimum_balance(spl_token::state::Account::get_packed_len())),
        spl_token::state::Account::get_packed_len() as u64,
        &spl_token::id(),
    );

    let (account_address, bump_seed) =
        Pubkey::find_program_address(token_account_address_seeds, program_id);

    if account_address != *token_account_info.key {
        msg!(
            "Create SPL Token Account with PDA: {:?} was requested while PDA: {:?} was expected",
            token_account_info.key,
            account_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let mut signers_seeds = token_account_address_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    invoke_signed(
        &create_account_instruction,
        &[
            payer_info.clone(),
            token_account_info.clone(),
            system_info.clone(),
        ],
        &[&signers_seeds[..]],
    )?;

    let initialize_account_instruction = spl_token::instruction::initialize_account(
        &spl_token::id(),
        token_account_info.key,
        token_mint_info.key,
        token_account_owner_info.key,
    )?;

    invoke(
        &initialize_account_instruction,
        &[
            payer_info.clone(),
            token_account_info.clone(),
            token_account_owner_info.clone(),
            token_mint_info.clone(),
            spl_token_info.clone(),
            rent_sysvar_info.clone(),
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
/*
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
} */

/// Transfers SPL Tokens
pub fn transfer_spl_tokens<'a>(
    source_info: &AccountInfo<'a>,
    destination_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    amount: u64,
    spl_token_info: &AccountInfo<'a>,
) -> ProgramResult {
    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        source_info.key,
        destination_info.key,
        authority_info.key,
        &[],
        amount,
    )
    .unwrap();

    invoke(
        &transfer_instruction,
        &[
            spl_token_info.clone(),
            authority_info.clone(),
            source_info.clone(),
            destination_info.clone(),
        ],
    )?;

    Ok(())
}

/// Transfers SPL Tokens from a token account owned by the provided PDA authority with seeds
pub fn transfer_spl_tokens_signed<'a>(
    source_info: &AccountInfo<'a>,
    destination_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    authority_seeds: &[&[u8]],
    program_id: &Pubkey,
    amount: u64,
    spl_token_info: &AccountInfo<'a>,
) -> ProgramResult {
    let (authority_address, bump_seed) = Pubkey::find_program_address(authority_seeds, program_id);

    if authority_address != *authority_info.key {
        msg!(
                "Transfer SPL Token with Authority PDA: {:?} was requested while PDA: {:?} was expected",
                authority_info.key,
                authority_address
            );
        return Err(ProgramError::InvalidSeeds);
    }

    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        source_info.key,
        destination_info.key,
        authority_info.key,
        &[],
        amount,
    )
    .unwrap();

    let mut signers_seeds = authority_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    invoke_signed(
        &transfer_instruction,
        &[
            spl_token_info.clone(),
            authority_info.clone(),
            source_info.clone(),
            destination_info.clone(),
        ],
        &[&signers_seeds[..]],
    )?;

    Ok(())
}
/*
/// Asserts the given account_info represents a valid SPL Token account which is initialized and belongs to spl_token program
pub fn assert_is_valid_spl_token_account(account_info: &AccountInfo) -> Result<(), ProgramError> {
    if account_info.data_is_empty() {
        return Err(SocialError::SplTokenAccountDoesNotExist.into());
    }

    if account_info.owner != &spl_token::id() {
        return Err(SocialError::SplTokenAccountWithInvalidOwner.into());
    }

    if account_info.data_len() != spl_token::state::Account::LEN {
        return Err(SocialError::SplTokenInvalidTokenAccountData.into());
    }

    // TokeAccount layout:   mint(32), owner(32), amount(8), delegate(36), state(1), ...
    let data = account_info.try_borrow_data()?;
    let state = array_ref![data, 108, 1];

    if state == &[0] {
        return Err(SocialError::SplTokenAccountNotInitialized.into());
    }

    Ok(())
}

/// Asserts the given mint_info represents a valid SPL Token Mint account  which is initialized and belongs to spl_token program
pub fn assert_is_valid_spl_token_mint(mint_info: &AccountInfo) -> Result<(), ProgramError> {
    if mint_info.data_is_empty() {
        return Err(SocialError::SplTokenMintDoesNotExist.into());
    }

    if mint_info.owner != &spl_token::id() {
        return Err(SocialError::SplTokenMintWithInvalidOwner.into());
    }

    if mint_info.data_len() != Mint::LEN {
        return Err(SocialError::SplTokenInvalidMintAccountData.into());
    }

    // In token program [36, 8, 1, is_initialized(1), 36] is the layout
    let data = mint_info.try_borrow_data().unwrap();
    let is_initialized = array_ref![data, 45, 1];

    if is_initialized == &[0] {
        return Err(SocialError::SplTokenMintNotInitialized.into());
    }

    Ok(())
}

/// Computationally cheap method to just get supply from a mint without unpacking the whole object
pub fn get_spl_token_mint_supply(mint_info: &AccountInfo) -> Result<u64, ProgramError> {
    assert_is_valid_spl_token_mint(mint_info)?;
    // In token program, 36, 8, 1, 1 is the layout, where the first 8 is supply u64.
    // so we start at 36.
    let data = mint_info.try_borrow_data().unwrap();
    let bytes = array_ref![data, 36, 8];

    Ok(u64::from_le_bytes(*bytes))
}

/// Computationally cheap method to get mint from a token account
/// It reads mint without deserializing full account data
pub fn get_spl_token_mint(token_account_info: &AccountInfo) -> Result<Pubkey, ProgramError> {
    assert_is_valid_spl_token_account(token_account_info)?;

    // TokeAccount layout:   mint(32), owner(32), amount(8), ...
    let data = token_account_info.try_borrow_data()?;
    let mint_data = array_ref![data, 0, 32];
    Ok(Pubkey::new_from_array(*mint_data))
}

/// Computationally cheap method to get owner from a token account
/// It reads owner without deserializing full account data
pub fn get_spl_token_owner(token_account_info: &AccountInfo) -> Result<Pubkey, ProgramError> {
    assert_is_valid_spl_token_account(token_account_info)?;

    // TokeAccount layout:   mint(32), owner(32), amount(8)
    let data = token_account_info.try_borrow_data()?;
    let owner_data = array_ref![data, 32, 32];
    Ok(Pubkey::new_from_array(*owner_data))
}

/// Returns Tokens balance.
/// Extrats balance field without unpacking entire struct.
pub fn get_token_balance(token_account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = token_account.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Account::get_packed_len())?;
    let amount = array_ref![data, 64, 8];

    Ok(u64::from_le_bytes(*amount))
}
 */
