//! General purpose account utility functions
#![allow(dead_code)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, msg, program::invoke,
    program::invoke_signed, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey, rent::Rent, system_instruction::create_account, system_program, sysvar::Sysvar,
};

use crate::{accounts::AccountContainer, error::AccountError};

/// Trait for accounts to return their max size
pub trait MaxSize {
    /// Returns max account size or None if max size is not known and actual instance size should be used
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}



/// Creates a new account and serializes data into it using AccountMaxSize to determine the account's size
pub fn create_and_serialize_account<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &AccountContainer,
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    // Assert the account is not initialized yet
    if !(account_info.data_is_empty() && *account_info.owner == system_program::id()) {
        return Err(AccountError::AccountAlreadyInitialized.into());
    }

    let (serialized_data, account_size) = if let Some(max_size) = account_data.get_max_size() {
        (None, max_size)
    } else {
        let serialized_data = account_data.try_to_vec()?;
        let account_size = serialized_data.len();
        (Some(serialized_data), account_size)
    };

    let rent = Rent::get()?;

    let create_account_instruction = create_account(
        payer_info.key,
        account_info.key,
        rent.minimum_balance(account_size),
        account_size as u64,
        program_id,
    );

    invoke(
        &create_account_instruction,
        &[
            payer_info.clone(),
            account_info.clone(),
            system_info.clone(),
        ],
    )?;
    if let Some(serialized_data) = serialized_data {
        account_info
            .data
            .borrow_mut()
            .copy_from_slice(&serialized_data);
    } else {
        account_data.serialize(&mut *account_info.data.borrow_mut())?;
    }

    Ok(())
}
/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_signed<'a>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &AccountContainer,
    account_address_seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let (account_address, bump_seed) =
        Pubkey::find_program_address(account_address_seeds, program_id);
    return create_and_serialize_account_signed_from_pda(payer_info, account_info, account_data, account_address_seeds, program_id, system_info, rent,account_address, bump_seed);
}


/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_signed_verify<'a >(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &AccountContainer,
    account_address_seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
    account_address:Pubkey, 
    bump_seed: u8
) -> Result<(), ProgramError> {
    
    let mut seeds_with_bump = account_address_seeds.to_vec();
    let bump_seeds = [bump_seed];
    seeds_with_bump.push(&bump_seeds);
    let (account_address_pda_debug, bump_debug) =
        Pubkey::find_program_address(account_address_seeds, program_id);

    msg!("-----");

    msg!(account_address_pda_debug.to_string().as_str());
    msg!(bump_debug.to_string().as_str());
    msg!(bump_seed.to_string().as_str());

    let account_address_pda =
        Pubkey::create_program_address(seeds_with_bump.as_slice(), program_id)?;
    if account_address != account_address_pda
    {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            account_address_pda,
            account_address
        );
        return Err(ProgramError::InvalidSeeds);
    }
    return create_and_serialize_account_signed_from_pda(payer_info, account_info, account_data, account_address_seeds, program_id, system_info, rent,account_address, bump_seed);
}


/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
fn create_and_serialize_account_signed_from_pda<'a, T: BorshSerialize + MaxSize >(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    account_address_seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
    account_address:Pubkey, 
    bump_seed: u8
) -> Result<(), ProgramError> {
    
    // Get PDA and assert it's the same as the requested account address
    if account_address != *account_info.key {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            account_info.key,
            account_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let (serialized_data, account_size) = if let Some(max_size) = account_data.get_max_size() {
        (None, max_size)
    } else {
        let serialized_data = account_data.try_to_vec()?;
        let account_size = serialized_data.len();
        (Some(serialized_data), account_size)
    };

    let create_account_instruction = create_account(
        payer_info.key,
        account_info.key,
        rent.minimum_balance(account_size),
        account_size as u64,
        program_id,
    );

    let mut signers_seeds = account_address_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    invoke_signed(
        &create_account_instruction,
        &[
            payer_info.clone(),
            account_info.clone(),
            system_info.clone(),
        ],
        &[&signers_seeds[..]],
    )?;

    if let Some(mut serialized_data) = serialized_data {
        account_info
            .data
            .borrow_mut()
            .copy_from_slice(&serialized_data);
    } else {
        account_data.serialize( &mut *account_info.data.borrow_mut())?;
        // account_info.data will be empty after this even though some value has been serialized
    }

    Ok(())
}




/// Deserializes account and checks it's initialized and owned by the specified program
pub fn get_account_data<T: BorshDeserialize + IsInitialized>(
    owner_program_id: &Pubkey,
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    if account_info.data_is_empty() {
        return Err(AccountError::AccountDoesNotExist.into());
    }
    if account_info.owner != owner_program_id {
        return Err(AccountError::InvalidAccountOwner.into());
    }

    let account: T = try_from_slice_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(ProgramError::UninitializedAccount)
    } else {
        Ok(account)
    }
}

/// Asserts the given account is not empty, owned by the given program and of the expected type
/// Note: The function assumes the account type T is stored as the first element in the account data
pub fn assert_is_valid_account<T: BorshDeserialize + PartialEq>(
    account_info: &AccountInfo,
    expected_account_type: T,
    owner_program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if account_info.owner != owner_program_id {
        return Err(AccountError::InvalidAccountOwner.into());
    }

    if account_info.data_is_empty() {
        return Err(AccountError::AccountDoesNotExist.into());
    }

    let account_type: T = try_from_slice_unchecked(&account_info.data.borrow())?;

    if account_type != expected_account_type {
        return Err(AccountError::InvalidAccountType.into());
    };

    Ok(())
}

/// Disposes account by transferring its lamports to the beneficiary account and zeros its data
// After transaction completes the runtime would remove the account with no lamports
pub fn dispose_account(account_info: &AccountInfo, beneficiary_info: &AccountInfo) {
    let account_lamports = account_info.lamports();
    **account_info.lamports.borrow_mut() = 0;

    **beneficiary_info.lamports.borrow_mut() = beneficiary_info
        .lamports()
        .checked_add(account_lamports)
        .unwrap();

    let mut account_data = account_info.data.borrow_mut();

    account_data.fill(0);
}