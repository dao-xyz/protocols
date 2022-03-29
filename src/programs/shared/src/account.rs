use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_memory::sol_memset;
use solana_program::rent::Rent;
use solana_program::system_instruction::create_account;
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, program_error::ProgramError,
    program_pack::IsInitialized, pubkey::Pubkey,
};

use solana_program::{msg, system_instruction, system_program};

use crate::error::UtilsError;

/// Checks if the slice has at least min_len size
pub fn check_data_len(data: &[u8], min_len: usize) -> Result<(), ProgramError> {
    if data.len() < min_len {
        Err(ProgramError::AccountDataTooSmall)
    } else {
        Ok(())
    }
}

pub trait MaxSize {
    /// Returns max account size or None if max size is not known and actual instance size should be used
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

/// Deserializes account and checks it's initialized and owned by the specified program
pub fn get_account_data<T: BorshDeserialize + IsInitialized>(
    owner_program_id: &Pubkey,
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    if account_info.data_is_empty() {
        return Err(UtilsError::AccountDoesNotExist.into());
    }
    if account_info.owner != owner_program_id {
        return Err(UtilsError::InvalidAccountOwner.into());
    }

    let account: T = try_from_slice_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(ProgramError::UninitializedAccount)
    } else {
        Ok(account)
    }
}

/// Check account owner is the given program
pub fn check_account_owner(
    account_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if *program_id != *account_info.owner {
        msg!(
            "Expected account to be owned by program {}, received {}",
            program_id,
            account_info.owner
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Check system program address
pub fn check_system_program(program_id: &Pubkey) -> Result<(), ProgramError> {
    if *program_id != system_program::id() {
        msg!(
            "Expected system program {}, received {}",
            system_program::id(),
            program_id
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_verify_with_bump<'a, T: BorshSerialize + MaxSize>(
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
    create_and_serialize_account_with_bump(
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
pub fn create_and_serialize_account_with_bump<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    check_system_program(&system_info.key)?;

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

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// The owner of the account is set to the PDA program
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_signed<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    account_address_seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    create_and_serialize_account_with_owner_signed(
        payer_info,
        account_info,
        account_data,
        account_address_seeds,
        program_id,
        program_id, // By default use PDA program_id as the owner of the account
        system_info,
        rent,
    )
}

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
#[allow(clippy::too_many_arguments)]
pub fn create_and_serialize_account_with_owner_signed<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    account_address_seeds: &[&[u8]],
    program_id: &Pubkey,
    owner_program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    // Get PDA and assert it's the same as the requested account address
    let (account_address, bump_seed) =
        Pubkey::find_program_address(account_address_seeds, program_id);

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

    let mut signers_seeds = account_address_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    let rent_exempt_lamports = rent.minimum_balance(account_size).max(1);

    // If the account has some lamports already it can't be created using create_account instruction
    // Anybody can send lamports to a PDA and by doing so create the account and perform DoS attack by blocking create_account
    if account_info.lamports() > 0 {
        let top_up_lamports = rent_exempt_lamports.saturating_sub(account_info.lamports());

        if top_up_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer_info.key, account_info.key, top_up_lamports),
                &[
                    payer_info.clone(),
                    account_info.clone(),
                    system_info.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(account_info.key, account_size as u64),
            &[account_info.clone(), system_info.clone()],
            &[&signers_seeds[..]],
        )?;

        invoke_signed(
            &system_instruction::assign(account_info.key, owner_program_id),
            &[account_info.clone(), system_info.clone()],
            &[&signers_seeds[..]],
        )?;
    } else {
        // If the PDA doesn't exist use create_account to use lower compute budget
        let create_account_instruction = create_account(
            payer_info.key,
            account_info.key,
            rent_exempt_lamports,
            account_size as u64,
            owner_program_id,
        );

        invoke_signed(
            &create_account_instruction,
            &[
                payer_info.clone(),
                account_info.clone(),
                system_info.clone(),
            ],
            &[&signers_seeds[..]],
        )?;
    }

    if let Some(serialized_data) = serialized_data {
        account_info
            .data
            .borrow_mut()
            .copy_from_slice(&serialized_data);
    } else if account_size > 0 {
        account_data.serialize(&mut *account_info.data.borrow_mut())?;
    }

    Ok(())
}

/// Asserts the given account is not empty, owned by the given program and one of the types asserted via the provided predicate function
/// Note: The function assumes the account type T is stored as the first element in the account data
pub fn assert_is_valid_account_of_types<T: BorshDeserialize + PartialEq, F: Fn(&T) -> bool>(
    owner_program_id: &Pubkey,
    account_info: &AccountInfo,
    is_account_type: F,
) -> Result<(), ProgramError> {
    if account_info.data_is_empty() {
        return Err(UtilsError::AccountDoesNotExist.into());
    }
    if account_info.owner != owner_program_id {
        return Err(UtilsError::InvalidAccountOwner.into());
    }

    let account_type: T = try_from_slice_unchecked(&account_info.data.borrow())?;

    if !is_account_type(&account_type) {
        return Err(UtilsError::InvalidAccountType.into());
    };

    Ok(())
}



/* pub fn close_system_account<'a, 'b>(
    receiving_account: &'a AccountInfo<'b>,
    target_account: &'a AccountInfo<'b>,
    max_size: usize,
    authority_account: &Pubkey,
) -> ProgramResult {
    if *target_account.owner != *authority_account {
        return Err(ProgramError::IllegalOwner);
    }
    let receiving_account_amount = receiving_account.lamports();
    **receiving_account.lamports.borrow_mut() = receiving_account_amount
        .checked_add(target_account.lamports())
        .unwrap();
    **target_account.lamports.borrow_mut() = 0;
    sol_memset(
        *target_account.data.borrow_mut(),
        0,
        max_size,
    );

    Ok(())
}
 */

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