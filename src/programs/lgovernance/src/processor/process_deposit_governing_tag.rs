//! Program state processor

use ltag::state::{get_tag_record_data_with_factory_and_signed_owner, TagRecordFactoryAccount};
use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::GovernanceError,
    state::{scopes::scope::VotePowerUnit, vote_power_origin_record::VotePowerOriginRecord},
};

/// Processes DepositGoverningTag instruction
pub fn process_deposit_governing_tag(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_origin_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let tag_record_info = next_account_info(accounts_iter)?;
    let tag_record_owner_info = next_account_info(accounts_iter)?;
    let tag_record_factory_info = next_account_info(accounts_iter)?;
    let token_origin_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    // check if tag
    let tag_record_data = get_tag_record_data_with_factory_and_signed_owner(
        &ltag::id(),
        tag_record_info,
        tag_record_factory_info.key,
        tag_record_owner_info,
    )?;

    let tag_record_factory =
        get_account_data::<TagRecordFactoryAccount>(&ltag::id(), tag_record_factory_info)?;

    if tag_record_factory.owner_can_transfer
        || &tag_record_data.factory != tag_record_factory_info.key
    {
        // For now don't allow transferable tags since they will mess up vote power
        // Since an owner could "deposit" its tag, then transfer it
        return Err(GovernanceError::InvalidTagRecordFactory.into());
    }

    VotePowerOriginRecord::create(
        program_id,
        VotePowerUnit::Tag {
            record_factory: *tag_record_factory_info.key,
        },
        1,
        &rent,
        token_origin_record_info,
        token_origin_record_bump_seed,
        tag_record_owner_info,
        payer_info,
        system_info,
    )?;
    Ok(())
}
