use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    stake, system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction::initialize_mint, state::Mint};

use crate::tokens::spl_utils::create_account_mint_account_seeds;

pub fn create_pool_mint<'a>(
    stake_pool: &Pubkey,
    mint_info: &AccountInfo<'a>,
    mint_bump_seed: u8,
    mint_authority: &Pubkey,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let mint_rent = rent.minimum_balance(Mint::LEN);

    /*   let (mint_authority, _) =
           spl_stake_pool::find_withdraw_authority_program_address(program_id, stake_pool);
    */
    let mint_bump_seeds = &[mint_bump_seed];
    let mint_account_seeds = ["mint".as_bytes(), mint_bump_seeds];
    let address = Pubkey::create_program_address(&mint_account_seeds, program_id).unwrap();
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
        &[&mint_account_seeds], // missing things here, we need the full seed for the mint accoutn
    )?;

    invoke(
        &initialize_mint(
            &spl_token::id(),
            mint_info.key,
            &mint_authority,
            None, //freeze_authority_pubkey.as_ref(),
            spl_token::native_mint::DECIMALS,
        )?,
        &[mint_info.clone(), rent_info.clone()],
    )?;
    Ok(())
}
pub fn create_independent_reserve_stake_account<'a>(
    reserve_stake_info: &AccountInfo<'a>,
    reserve_stake_account_bump_seed: u8,
    stake_pool: &Pubkey,
    payer_info: &AccountInfo<'a>,
    withdraw_authority: &Pubkey,
    stake_program_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    create_independent_stake_account(
        reserve_stake_info,
        &[
            "reserve_stake".as_bytes(),
            &stake_pool.to_bytes(),
            &[reserve_stake_account_bump_seed],
        ],
        payer_info,
        &stake::state::Authorized {
            staker: *withdraw_authority,
            withdrawer: *withdraw_authority,
        },
        &stake::state::Lockup::default(),
        1,
        stake_program_info,
        rent_info,
        program_id,
    )
}
pub fn create_independent_stake_account<'a>(
    stake_info: &AccountInfo<'a>,
    stake_account_bump_seeds: &[&[u8]],
    payer_info: &AccountInfo<'a>,
    authorized: &stake::state::Authorized,
    lockup: &stake::state::Lockup,
    stake_amount: u64,
    stake_program_info: &AccountInfo<'a>,

    rent_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let stake_account_size = std::mem::size_of::<stake::state::StakeState>();
    let stake_rent = rent.minimum_balance(stake_account_size) + stake_amount;

    let address = Pubkey::create_program_address(&stake_account_bump_seeds, program_id).unwrap();
    if stake_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            stake_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            stake_info.key,
            stake_rent,
            stake_account_size as u64,
            &stake::program::id(),
        ),
        &[payer_info.clone(), stake_info.clone()],
        &[stake_account_bump_seeds],
    )?;

    invoke(
        &stake::instruction::initialize(stake_info.key, authorized, lockup),
        &[
            stake_info.clone(),
            rent_info.clone(),
            stake_program_info.clone(),
        ],
    )?;
    /*    let instructions = stake::instruction::create_account(
           payer_info.key,
           stake_info.key,
           authorized,
           lockup,
           stake_rent,
       );
    */

    Ok(())
}
