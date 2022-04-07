use borsh::BorshSerialize;
use shared::{
    account::{create_and_serialize_account_verify_with_bump, get_account_data},
    content::ContentSource,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    accounts::AccountType,
    shared::names::entity_name_is_valid,
    state::{
        channel::{get_channel_account_program_address_seeds, ChannelAccount},
        channel_authority::{
            check_activity_authority, get_channel_authority_address_seed, AuthorityCondition,
            AuthorityType, ChannelAuthority,
        },
    },
};

pub fn process_create_channel(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    parent: Option<Pubkey>,
    name: String,
    link: Option<ContentSource>,
    channel_account_bump_seed: u8,
    authority_seed: Pubkey,
    authority_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let channel_account_info = next_account_info(accounts_iter)?;
    let creator = next_account_info(accounts_iter)?;
    let manage_authority_authority_record_info = next_account_info(accounts_iter)?;
    let manage_authority_authority_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    if !creator.is_signer {
        // Do not let someone create an channel for someone else without their signature
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !manage_authority_authority_info.is_signer {
        // Make sure creator can sign authority
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !entity_name_is_valid(name.as_ref()) {
        return Err(ProgramError::InvalidArgument);
    }

    if !channel_account_info.try_data_is_empty()? {
        // Channel already exist
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(_parent) = parent {
        let parent_channel_account_info = next_account_info(accounts_iter)?;
        let parent_channel =
            get_account_data::<ChannelAccount>(program_id, parent_channel_account_info)?;

        let authority_info = next_account_info(accounts_iter)?;

        check_activity_authority(
            program_id,
            authority_info,
            &AuthorityType::CreateSubChannel,
            &parent_channel,
            parent_channel_account_info,
            accounts_iter,
        )?;
    }

    let rent = Rent::get()?;
    /*
       Channel and user names must be unique, as we generate the seeds in the same way for both
       Do we want this really?
    */
    let mut seeds = get_channel_account_program_address_seeds(name.as_ref())?;
    seeds.push(vec![channel_account_bump_seed]);
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    create_and_serialize_account_verify_with_bump(
        payer_account,
        channel_account_info,
        &ChannelAccount {
            account_type: AccountType::Channel,
            creator: *creator.key,
            parent,
            link,
            name,
            creation_timestamp: Clock::get()?.unix_timestamp as u64,
            /* authority: Some(*authority.key), */
        },
        seed_slice,
        program_id,
        system_account,
        &rent,
    )?;

    // Create admin authority
    create_and_serialize_account_verify_with_bump(
        payer_account,
        manage_authority_authority_record_info,
        &ChannelAuthority {
            account_type: AccountType::ChannelAuthority,
            authority_types: vec![AuthorityType::Admin],
            channel: *channel_account_info.key,
            condition: AuthorityCondition::Pubkey(*manage_authority_authority_info.key),
            seed: authority_seed,
        },
        &get_channel_authority_address_seed(
            channel_account_info.key,
            &authority_seed,
            &[authority_bump_seed],
        ),
        program_id,
        system_account,
        &rent,
    )?;

    Ok(())
}

pub fn process_update_channel_info(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    link: Option<ContentSource>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let channel_account_info = next_account_info(accounts_iter)?;
    let mut channel = get_account_data::<ChannelAccount>(program_id, channel_account_info)?;
    let authority_info = next_account_info(accounts_iter)?;

    check_activity_authority(
        program_id,
        authority_info,
        &AuthorityType::ManageInfo,
        &channel,
        channel_account_info,
        accounts_iter,
    )?;

    channel.link = link;
    channel.serialize(&mut *channel_account_info.data.borrow_mut())?;
    Ok(())
}
