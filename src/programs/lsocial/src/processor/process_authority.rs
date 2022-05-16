use crate::{
    accounts::AccountType,
    error::SocialError,
    state::{
        channel::ChannelAccount,
        channel_authority::{
            check_activity_authority, get_channel_authority_address_seed,
            get_channel_authority_data_for_channel, AuthorityCondition, AuthorityType,
            ChannelAuthority,
        },
    },
};

use shared::account::{
    check_system_program, create_and_serialize_account_verify_with_bump, dispose_account,
    get_account_data,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    authority_types: Vec<AuthorityType>,
    condition: AuthorityCondition,
    seed: Pubkey,
    authority_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let new_channel_authority_info = next_account_info(accounts_iter)?;
    let channel_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    check_system_program(system_account.key)?;

    let channel_data = get_account_data::<ChannelAccount>(program_id, channel_info)?;

    let authority_info = next_account_info(accounts_iter)?;

    check_activity_authority(
        program_id,
        authority_info,
        &AuthorityType::Admin,
        &channel_data,
        channel_info,
        accounts_iter,
    )?;

    if authority_types.len() == 0 {
        // Expecting atleast 1 authority type
        return Err(SocialError::InvalidAuthorityType.into());
    }
    let rent = Rent::get()?;
    create_and_serialize_account_verify_with_bump(
        payer_account,
        new_channel_authority_info,
        &ChannelAuthority {
            account_type: AccountType::ChannelAuthority,
            authority_types,
            channel: *channel_info.key,
            condition,
            seed,
        },
        &get_channel_authority_address_seed(channel_info.key, &seed, &[authority_bump_seed]),
        program_id,
        system_account,
        &rent,
    )?;

    Ok(())
}

pub fn process_delete_authority(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let delete_authority_info = next_account_info(accounts_iter)?;
    let channel_info = next_account_info(accounts_iter)?;
    let beneficiary_info = next_account_info(accounts_iter)?;
    let channel_data = get_account_data::<ChannelAccount>(program_id, channel_info)?;

    let authority_info = next_account_info(accounts_iter)?;

    if delete_authority_info.key == authority_info.key {
        return Err(SocialError::InvalidAuthority.into());
    }

    check_activity_authority(
        program_id,
        authority_info,
        &AuthorityType::Admin,
        &channel_data,
        channel_info,
        accounts_iter,
    )?;

    // Check that the authority we want to delete adheres to this channel
    let _delete_authority_data = get_channel_authority_data_for_channel(
        program_id,
        delete_authority_info,
        channel_info.key,
    )?;

    dispose_account(delete_authority_info, beneficiary_info);

    Ok(())
}
