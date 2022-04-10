use crate::{
    accounts::AccountType,
    error::SocialError,
    state::{
        channel::ChannelAccount,
        channel_authority::{check_activity_authority, AuthorityType},
        post::get_post_data,
        vote_record::Vote,
    },
    state::{
        post::{PostAccount, VoteConfig},
        vote_record::{
            get_vote_record_address_seeds, get_vote_record_data_for_signed_owner, VoteRecord,
        },
    },
};
use borsh::BorshSerialize;

use shared::account::{
    check_system_program, create_and_serialize_account_verify_with_bump, dispose_account,
    get_account_data,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_post_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote: Vote,
    vote_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let channel_info = next_account_info(accounts_iter)?;
    let vote_record_info = next_account_info(accounts_iter)?;
    let vote_record_owner_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;

    check_system_program(system_info.key)?;

    if !vote_record_info.data_is_empty() {
        return Err(SocialError::VoteAlreadyExist.into());
    }

    if !vote_record_owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let channel_data = get_account_data::<ChannelAccount>(program_id, channel_info)?;
    let mut post = get_post_data(program_id, post_account_info, channel_info.key)?;

    let authority_info = next_account_info(accounts_iter)?;

    check_activity_authority(
        program_id,
        authority_info,
        &AuthorityType::Vote,
        &channel_data,
        channel_info,
        accounts_iter,
    )?;

    let rent = Rent::get()?;
    let bump_seeds = [vote_record_bump_seed];
    let seeds = get_vote_record_address_seeds(
        post_account_info.key,
        vote_record_owner_info.key,
        &bump_seeds,
    );
    create_and_serialize_account_verify_with_bump(
        payer_info,
        vote_record_info,
        &VoteRecord {
            account_type: AccountType::VoteRecord,
            owner: *vote_record_owner_info.key,
            post: *post_account_info.key,
            vote: vote.clone(),
        },
        &seeds,
        program_id,
        system_info,
        &rent,
    )?;

    post.vote_config = match post.vote_config {
        VoteConfig::Simple { downvote, upvote } => match &vote {
            Vote::Up => VoteConfig::Simple {
                downvote,
                upvote: upvote.checked_add(1).unwrap(),
            },
            Vote::Down => VoteConfig::Simple {
                downvote: downvote.checked_add(1).unwrap(),
                upvote,
            },
        },
    };
    post.serialize(&mut *post_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_post_unvote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let channel_info = next_account_info(accounts_iter)?;
    let vote_record_info = next_account_info(accounts_iter)?;
    let vote_record_owner_info = next_account_info(accounts_iter)?;
    let destination_info = next_account_info(accounts_iter)?;

    let authority_info = next_account_info(accounts_iter)?;
    let channel_data = get_account_data::<ChannelAccount>(program_id, channel_info)?;
    check_activity_authority(
        program_id,
        authority_info,
        &AuthorityType::Vote,
        &channel_data,
        channel_info,
        accounts_iter,
    )?;

    let mut post = get_account_data::<PostAccount>(program_id, post_account_info)?;
    if vote_record_info.data_is_empty() {
        return Err(SocialError::VoteDoesNotExist.into());
    }

    let vote_record_data = get_vote_record_data_for_signed_owner(
        program_id,
        vote_record_info,
        vote_record_owner_info,
    )?;

    msg!("DISPOSE {}", vote_record_info.key);
    dispose_account(vote_record_info, destination_info);

    post.vote_config = match post.vote_config {
        VoteConfig::Simple { downvote, upvote } => match &vote_record_data.vote {
            Vote::Up => VoteConfig::Simple {
                downvote,
                upvote: upvote.checked_sub(1).unwrap(),
            },
            Vote::Down => VoteConfig::Simple {
                downvote: downvote.checked_sub(1).unwrap(),
                upvote,
            },
        },
    };
    post.serialize(&mut *post_account_info.data.borrow_mut())?;

    Ok(())
}
