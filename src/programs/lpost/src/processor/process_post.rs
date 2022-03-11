use crate::{
    accounts::AccountType,
    error::PostError,
    instruction::{CreatePost, CreateVoteConfig},
    state::{
        assert_authorized_by_tag,
        post::{PostAccount, VoteConfig},
    },
};
use lchannel::state::{ChannelAccount, ChannelAuthority};

use shared::account::{
    check_system_program, create_and_serialize_account_verify_with_bump,
    get_account_data,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_post(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    post: CreatePost,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    if !post_account_info.data_is_empty() {
        return Err(PostError::PostAlreadyExist.into());
    }
    let channel_account_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let content_hash = post.hash;

    let channel_data = get_account_data::<ChannelAccount>(&lchannel::id(), channel_account_info)?;
    check_system_program(system_account.key)?;

    match &channel_data.channel_authority_config {
        ChannelAuthority::AuthorityByTag { tag, authority } => {
            let tag_record_info = next_account_info(accounts_iter)?;
            let tag_authority_info = next_account_info(accounts_iter)?;
            let tag_owner_info = next_account_info(accounts_iter)?;
            if authority != tag_authority_info.key {
                return Err(PostError::InvaligTagAuthority.into());
            }
            assert_authorized_by_tag(tag_owner_info, tag_record_info, tag, tag_authority_info)?;
        }
    }

    let rent = Rent::get()?;
    let clock = Clock::get()?;
    create_and_serialize_account_verify_with_bump(
        payer_account,
        post_account_info,
        &PostAccount {
            create_at_timestamp: clock.unix_timestamp,
            deleted_at_timestamp: None,
            account_type: AccountType::Post,
            content: post.content,
            channel: *channel_account_info.key,
            hash: post.hash,
            creator: *payer_account.key,
            vote_config: match post.vote_config {
                CreateVoteConfig::Simple => VoteConfig::Simple {
                    upvote: 0,
                    downvote: 0,
                },
            },
        },
        &[&content_hash, &[post.post_bump_seed]],
        program_id,
        system_account,
        &rent,
    )?;

    Ok(())
}
