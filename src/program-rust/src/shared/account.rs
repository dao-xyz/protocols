//! General purpose account utility functions
#![allow(dead_code)]

use std::cmp::Ordering;

use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    program::invoke, program::invoke_signed, program_error::ProgramError,
    program_pack::IsInitialized, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction::create_account, system_program, sysvar::Sysvar,
};
use spl_token::state::Mint;

use crate::{
    error::AccountError, math, social::accounts::AccountContainer, tokens::pack::check_data_len,
};

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

    if account_info.key != &account_address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            account_address,
            account_info.key
        );
        return Err(ProgramError::InvalidSeeds);
    }
    create_and_serialize_account_signed_verify_with_bump(
        payer_info,
        account_info,
        account_data,
        account_address_seeds,
        program_id,
        system_info,
        rent,
        bump_seed,
    )
}

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_signed_verify_with_bump<'a>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &AccountContainer,
    account_address_seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
    bump_seed: u8,
) -> Result<(), ProgramError> {
    let mut seeds_with_bump = account_address_seeds.to_vec();
    let bump_seeds = [bump_seed];
    seeds_with_bump.push(&bump_seeds);

    create_and_serialize_account_signed_verify(
        payer_info,
        account_info,
        account_data,
        seeds_with_bump.as_slice(),
        program_id,
        system_info,
        rent,
    )
}

/// Creates a new account and serializes data into it using the provided seeds to invoke signed CPI call
/// Note: This functions also checks the provided account PDA matches the supplied seeds
pub fn create_and_serialize_account_signed_verify<'a>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &AccountContainer,
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
    create_and_serialize_account_signed_from_pda(
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
fn create_and_serialize_account_signed_from_pda<'a, T: BorshSerialize + MaxSize>(
    payer_info: &AccountInfo<'a>,
    account_info: &AccountInfo<'a>,
    account_data: &T,
    seeds: &[&[u8]],
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
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
    invoke_signed(
        &create_account_instruction,
        &[
            payer_info.clone(),
            account_info.clone(),
            system_info.clone(),
        ],
        &[&seeds[..]],
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
/* pub fn dispose_account(account_info: &AccountInfo, beneficiary_info: &AccountInfo) {
    let account_lamports = account_info.lamports();
    **account_info.lamports.borrow_mut() = 0;
    **beneficiary_info.lamports.borrow_mut() = beneficiary_info
        .lamports()
        .checked_add(account_lamports)
        .unwrap();

    let mut account_data = account_info.data.borrow_mut();

    account_data.fill(0);
}
 */

/// Returns Token Mint supply.
/// Extrats supply field without unpacking entire struct.
pub fn get_token_supply(token_mint: &AccountInfo) -> Result<u64, ProgramError> {
    let data = token_mint.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Mint::get_packed_len())?;
    let supply = array_ref![data, 36, 8];

    Ok(u64::from_le_bytes(*supply))
}

/// Returns Token decimals.
/// Extrats decimals field without unpacking entire struct.
pub fn get_token_decimals(token_mint: &AccountInfo) -> Result<u8, ProgramError> {
    let data = token_mint.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Mint::get_packed_len())?;
    let decimals = array_ref![data, 44, 1];

    Ok(decimals[0])
}

/// Returns Tokens balance.
/// Extrats balance field without unpacking entire struct.
pub fn get_token_balance(token_account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = token_account.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Account::get_packed_len())?;
    let amount = array_ref![data, 64, 8];

    Ok(u64::from_le_bytes(*amount))
}

/// Returns Token account owner.
/// Extrats owner field without unpacking entire struct.
pub fn get_token_account_owner(token_account: &AccountInfo) -> Result<Pubkey, ProgramError> {
    let data = token_account.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Account::get_packed_len())?;
    let owner = array_ref![data, 32, 32];

    Ok(Pubkey::new_from_array(*owner))
}

/// Returns Token account mint.
/// Extrats mint field without unpacking entire struct.
pub fn get_token_account_mint(token_account: &AccountInfo) -> Result<Pubkey, ProgramError> {
    let data = token_account.try_borrow_data()?;
    check_data_len(&data, spl_token::state::Account::get_packed_len())?;
    let mint = array_ref![data, 0, 32];

    Ok(Pubkey::new_from_array(*mint))
}

pub fn get_balance_increase(
    account: &AccountInfo,
    previous_balance: u64,
) -> Result<u64, ProgramError> {
    let balance = get_token_balance(account)?;
    if balance >= previous_balance {
        Ok(balance - previous_balance)
    } else {
        msg!(
            "Error: Balance decrease was not expected. Account: {}",
            account.key
        );
        Err(ProgramError::Custom(1001))
    }
}

pub fn get_balance_decrease(
    account: &AccountInfo,
    previous_balance: u64,
) -> Result<u64, ProgramError> {
    let balance = get_token_balance(account)?;
    if balance <= previous_balance {
        Ok(previous_balance - balance)
    } else {
        msg!(
            "Error: Balance increase was not expected. Account: {}",
            account.key
        );
        Err(ProgramError::Custom(1002))
    }
}

pub fn check_tokens_spent(
    account: &AccountInfo,
    previous_balance: u64,
    max_amount_spent: u64,
) -> Result<u64, ProgramError> {
    let tokens_spent = get_balance_decrease(account, previous_balance)?;
    if tokens_spent > max_amount_spent {
        msg!(
            "Error: Invoked program overspent. Account: {}, max expected: {}, actual: {}",
            account.key,
            max_amount_spent,
            tokens_spent
        );
        Err(ProgramError::Custom(1003))
    } else {
        Ok(tokens_spent)
    }
}

pub fn check_tokens_received(
    account: &AccountInfo,
    previous_balance: u64,
    min_amount_received: u64,
) -> Result<u64, ProgramError> {
    let tokens_received = get_balance_increase(account, previous_balance)?;
    if tokens_received < min_amount_received {
        msg!(
            "Error: Not enough tokens returned by invoked program. Account: {}, min expected: {}, actual: {}",
            account.key,
            min_amount_received,
            tokens_received
        );
        Err(ProgramError::Custom(1004))
    } else {
        Ok(tokens_received)
    }
}

/// Returns Token Mint data.
pub fn get_token_mint(token_mint: &AccountInfo) -> Result<Mint, ProgramError> {
    let data = token_mint.try_borrow_data()?;
    Mint::unpack(&data)
}

/// Returns Token Account data.
pub fn get_token_account(
    token_account: &AccountInfo,
) -> Result<spl_token::state::Account, ProgramError> {
    let data = token_account.try_borrow_data()?;
    spl_token::state::Account::unpack(&data)
}

/// Returns token pair ratio, optimized for on-chain.
pub fn get_token_ratio<'a, 'b>(
    token_a_balance: u64,
    token_b_balance: u64,
    token_a_mint: &'a AccountInfo<'b>,
    token_b_mint: &'a AccountInfo<'b>,
) -> Result<f64, ProgramError> {
    get_token_ratio_with_decimals(
        token_a_balance,
        token_b_balance,
        get_token_decimals(token_a_mint)?,
        get_token_decimals(token_b_mint)?,
    )
}

/// Returns token pair ratio, uses decimals insted of mints, optimized for on-chain.
pub fn get_token_ratio_with_decimals(
    token_a_balance: u64,
    token_b_balance: u64,
    token_a_decimals: u8,
    token_b_decimals: u8,
) -> Result<f64, ProgramError> {
    if token_a_balance == 0 || token_b_balance == 0 {
        return Ok(0.0);
    }

    let mut ratio = token_b_balance as f64 / token_a_balance as f64;
    match token_a_decimals.cmp(&token_b_decimals) {
        Ordering::Greater => {
            for _ in 0..(token_a_decimals - token_b_decimals) {
                ratio *= 10.0;
            }
        }
        Ordering::Less => {
            for _ in 0..(token_b_decimals - token_a_decimals) {
                ratio /= 10.0;
            }
        }
        Ordering::Equal => {}
    }

    Ok(ratio)
}

/// Returns token pair ratio
pub fn get_token_pair_ratio<'a, 'b>(
    token_a_account: &'a AccountInfo<'b>,
    token_b_account: &'a AccountInfo<'b>,
) -> Result<f64, ProgramError> {
    let token_a_balance = get_token_balance(token_a_account)?;
    let token_b_balance = get_token_balance(token_b_account)?;
    if token_a_balance == 0 || token_b_balance == 0 {
        return Ok(0.0);
    }
    Ok(token_b_balance as f64 / token_a_balance as f64)
}

pub fn to_ui_amount(amount: u64, decimals: u8) -> f64 {
    let mut ui_amount = amount;
    for _ in 0..decimals {
        ui_amount /= 10;
    }
    ui_amount as f64
}

pub fn to_token_amount(ui_amount: f64, decimals: u8) -> Result<u64, ProgramError> {
    let mut amount = ui_amount;
    for _ in 0..decimals {
        amount *= 10.0;
    }
    math::checked_as_u64(amount)
}

pub fn to_amount_with_new_decimals(
    amount: u64,
    original_decimals: u8,
    new_decimals: u8,
) -> Result<u64, ProgramError> {
    match new_decimals.cmp(&original_decimals) {
        Ordering::Greater => {
            let mut new_amount = amount as f64;
            for _ in 0..(new_decimals - original_decimals) {
                new_amount *= 10.0;
            }
            math::checked_as_u64(new_amount)
        }
        Ordering::Less => {
            let mut new_amount = amount;
            for _ in 0..(original_decimals - new_decimals) {
                new_amount /= 10;
            }
            Ok(new_amount)
        }
        Ordering::Equal => Ok(amount),
    }
}

pub fn transfer_tokens<'a, 'b>(
    source_account: &'a AccountInfo<'b>,
    destination_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    amount: u64,
) -> ProgramResult {
    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            source_account.key,
            destination_account.key,
            authority_account.key,
            &[],
            amount,
        )?,
        &[
            source_account.clone(),
            destination_account.clone(),
            authority_account.clone(),
        ],
    )?;
    Ok(())
}

pub fn burn_tokens<'a, 'b>(
    from_token_account: &'a AccountInfo<'b>,
    mint_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    amount: u64,
) -> ProgramResult {
    invoke(
        &spl_token::instruction::burn(
            &spl_token::id(),
            from_token_account.key,
            mint_account.key,
            authority_account.key,
            &[],
            amount,
        )?,
        &[
            from_token_account.clone(),
            mint_account.clone(),
            authority_account.clone(),
        ],
    )
}

pub fn close_system_account<'a, 'b>(
    receiving_account: &'a AccountInfo<'b>,
    target_account: &'a AccountInfo<'b>,
    authority_account: &Pubkey,
) -> ProgramResult {
    if *target_account.owner != *authority_account {
        return Err(ProgramError::IllegalOwner);
    }
    let cur_balance = target_account.try_lamports()?;
    **receiving_account.try_borrow_mut_lamports()? += cur_balance;
    **target_account.try_borrow_mut_lamports()? -= cur_balance;

    if target_account.data_len() > 1000 {
        target_account.try_borrow_mut_data()?[..1000].fill(0);
    } else {
        target_account.try_borrow_mut_data()?.fill(0);
    }

    Ok(())
}

pub fn close_token_account<'a, 'b>(
    receiving_account: &'a AccountInfo<'b>,
    target_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
) -> ProgramResult {
    invoke(
        &spl_token::instruction::close_account(
            &spl_token::id(),
            receiving_account.key,
            target_account.key,
            authority_account.key,
            &[],
        )?,
        &[
            target_account.clone(),
            receiving_account.clone(),
            authority_account.clone(),
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spl_token::state::{Account, Mint};

    #[test]
    fn test_mint_supply_offset() {
        let mint = Mint {
            supply: 1234567891011,
            ..Mint::default()
        };
        let mut packed: [u8; 82] = [0; 82];
        Mint::pack(mint, &mut packed).unwrap();

        let supply = array_ref![packed, 36, 8];
        assert_eq!(1234567891011, u64::from_le_bytes(*supply));
    }

    #[test]
    fn test_mint_decimals_offset() {
        let mint = Mint {
            decimals: 123,
            ..Mint::default()
        };
        let mut packed: [u8; 82] = [0; 82];
        Mint::pack(mint, &mut packed).unwrap();

        let decimals = array_ref![packed, 44, 1];
        assert_eq!(123, decimals[0]);
    }

    #[test]
    fn test_account_amount_offset() {
        let account = Account {
            amount: 1234567891011,
            ..Account::default()
        };
        let mut packed: [u8; 165] = [0; 165];
        Account::pack(account, &mut packed).unwrap();

        let amount = array_ref![packed, 64, 8];
        assert_eq!(1234567891011, u64::from_le_bytes(*amount));
    }
}
