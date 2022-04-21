use crate::{
    accounts::AccountType,
    error::SocialError,
    instruction::CreateVoteConfig,
    state::{
        channel::{ChannelAccount, ChannelType},
        channel_authority::{check_activity_authority, AuthorityType},
        post::{
            get_post_data, get_post_program_address_seeds, PostAccount, PostContent, VoteConfig,
        },
    },
};

use shared::account::{
    check_system_program, create_and_serialize_account_verify_with_bump, get_account_data,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_post(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    content: PostContent,
    hash: [u8; 32],
    is_child: bool,
    vote_config: CreateVoteConfig,
    post_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let channel_info = next_account_info(accounts_iter)?;
    let owner_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    let parent = if is_child {
        let parent_post_info = next_account_info(accounts_iter)?;
        let _parent_post = get_post_data(program_id, parent_post_info, channel_info.key)?;
        *parent_post_info.key
    } else {
        *channel_info.key
    };

    if !post_account_info.data_is_empty() {
        return Err(SocialError::PostAlreadyExist.into());
    }

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }

    let channel_data = get_account_data::<ChannelAccount>(program_id, channel_info)?;
    check_system_program(system_account.key)?;

    if channel_data.channel_type == ChannelType::Collection {
        return Err(SocialError::InvalidChannelType.into());
    }

    let authority_info = next_account_info(accounts_iter)?;

    check_activity_authority(
        program_id,
        authority_info,
        &AuthorityType::CreatePost,
        &channel_data,
        channel_info,
        accounts_iter,
    )?;

    let rent = Rent::get()?;
    let clock = Clock::get()?;
    let bump_seeds = &[post_bump_seed];
    create_and_serialize_account_verify_with_bump(
        payer_account,
        post_account_info,
        &PostAccount {
            create_at_timestamp: clock.unix_timestamp,
            deleted_at_timestamp: None,
            account_type: AccountType::Post,
            content,
            channel: *channel_info.key,
            hash,
            creator: *owner_info.key,
            parent,
            vote_config: match vote_config {
                CreateVoteConfig::Simple => VoteConfig::Simple {
                    upvote: 0,
                    downvote: 0,
                },
            },
        },
        &get_post_program_address_seeds(&hash, bump_seeds),
        program_id,
        system_account,
        &rent,
    )?;

    Ok(())
}
