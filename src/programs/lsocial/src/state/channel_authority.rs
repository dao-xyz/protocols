use std::slice::Iter;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use ltag::state::{get_tag_record_data_with_factory_and_signed_owner, TagRecordAccount};
use shared::account::{get_account_data, MaxSize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    msg,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::{
    accounts::AccountType, error::SocialError,
    processor::utils::verify_signed_owner_maybe_sign_for_me,
};

use super::channel::ChannelAccount;

const AUTHORITY_SEED: &[u8] = b"authority";

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AuthorityType {
    Admin,
    CreatePost,
    DeleteAnyPost,
    Vote,
    Comment,
    ManageInfo,
    CreateChannel,
    RemoveChannel,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AuthorityCondition {
    Pubkey(Pubkey),
    Tag {
        #[allow(dead_code)] // but it's not
        record_factory: Pubkey,
    },
    None,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct ChannelAuthority {
    pub account_type: AccountType,
    pub channel: Pubkey,
    pub authority_types: Vec<AuthorityType>,
    pub condition: AuthorityCondition,
    pub seed: Pubkey,
}

impl MaxSize for ChannelAuthority {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for ChannelAuthority {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::ChannelAuthority
    }
}

pub fn check_activity_authority<'a>(
    program_id: &Pubkey,
    authority_info: &AccountInfo,
    required_authority_type: &AuthorityType,
    _channel: &ChannelAccount,
    channel_info: &AccountInfo<'a>,
    accounts_iter: &mut Iter<AccountInfo<'a>>,
) -> Result<(), ProgramError> {
    let authority_data =
        get_channel_authority_data_for_channel(program_id, authority_info, channel_info.key)?;

    // Check if user can use authority

    /// TODO limit capability of sign for me with scope
    ///
    match &authority_data.condition {
        AuthorityCondition::Tag { record_factory } => {
            let tag_record_info = next_account_info(accounts_iter)?;
            let tag_owner_info = next_account_info(accounts_iter)?;
            let data = get_account_data::<TagRecordAccount>(&ltag::id(), tag_record_info)?;

            if &data.factory != record_factory && &data.owner != tag_owner_info.key {
                return Err(SocialError::InvalidAuthority.into());
            }

            verify_signed_owner_maybe_sign_for_me(tag_owner_info, accounts_iter)?;
        }
        AuthorityCondition::Pubkey(pubkey) => {
            let signer = next_account_info(accounts_iter)?;
            if signer.key != pubkey {
                return Err(SocialError::InvalidAuthority.into());
            }
            verify_signed_owner_maybe_sign_for_me(signer, accounts_iter)?;
        }
        AuthorityCondition::None => {}
    }

    // Check if required authority type exist in the set
    if !authority_data
        .authority_types
        .contains(required_authority_type)
    {
        // Admin can do anything, but if not admin
        if !authority_data
            .authority_types
            .contains(&AuthorityType::Admin)
        {
            return Err(SocialError::InvalidAuthorityType.into());
        }
    }
    Ok(())
}

fn assert_authorized_by_tag<'a>(
    tag_record_info: &AccountInfo<'a>,
    tag_owner_info: &AccountInfo<'a>, //
    tag_record_factory: &Pubkey,
) -> Result<(), ProgramError> {
    if !tag_owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let _tag_record_data = get_tag_record_data_with_factory_and_signed_owner(
        &ltag::id(),
        tag_record_info,
        tag_record_factory,
        tag_owner_info,
    )?;

    Ok(())
}

pub fn get_channel_authority_data_for_channel(
    program_id: &Pubkey,
    channel_authority_info: &AccountInfo,
    channel: &Pubkey,
) -> Result<ChannelAuthority, ProgramError> {
    let data = get_account_data::<ChannelAuthority>(program_id, channel_authority_info)?;
    if &data.channel != channel {
        return Err(SocialError::InvalidChannelForAuthority.into());
    }
    Ok(data)
}

/// Returns Authority PDA seeds
pub fn get_channel_authority_address_seed<'a>(
    channel: &'a Pubkey,
    seed: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [AUTHORITY_SEED, channel.as_ref(), seed.as_ref(), bump_seed]
}

/// Returns VoteRecord PDA address
pub fn get_channel_authority_address(
    program_id: &Pubkey,
    channel: &Pubkey,
    seed: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[AUTHORITY_SEED, channel.as_ref(), seed.as_ref()],
        program_id,
    )
}
