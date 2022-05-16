use crate::state::{scopes::scope::VotePowerUnit, vote_power_owner_record::VotePowerOwnerRecord};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_delegatee(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    scope: Pubkey,
    vote_power_unit: VotePowerUnit,
    token_owner_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governing_owner_info = next_account_info(accounts_iter)?;
    let vote_power_owner_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    VotePowerOwnerRecord::create_empty_delegate(
        program_id,
        &scope,
        &rent,
        vote_power_owner_record_info,
        token_owner_record_bump_seed,
        governing_owner_info,
        &vote_power_unit,
        payer_info,
        system_info,
    )?;
    Ok(())
}
