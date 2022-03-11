use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, program_error::ProgramError,
    program_pack::IsInitialized, pubkey::Pubkey,
};

use crate::error::TagError;
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AccountType {
    Tag,
    TagRecord,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TagAccount {
    pub account_type: AccountType,
    pub tag: String,
}

impl MaxSize for TagAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for TagAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Tag
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TagRecordAccount {
    pub account_type: AccountType,
    pub tag: Pubkey,
    pub owner: Pubkey,
    pub authority: Pubkey,
}

impl MaxSize for TagRecordAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 32 + 32 + 32)
    }
}

impl IsInitialized for TagRecordAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::TagRecord
    }
}
///
pub fn get_tag_record_data_with_authority_and_signed_owner<'a>(
    program_id: &Pubkey,
    tag_record_info: &AccountInfo<'a>,
    authority: &Pubkey,
    owner: &AccountInfo<'a>,
) -> Result<TagRecordAccount, ProgramError> {
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<TagRecordAccount>(program_id, tag_record_info)?;

    if &data.authority != authority {
        return Err(TagError::InvalidAuthority.into());
    }

    if &data.owner != owner.key {
        return Err(TagError::InvalidOwner.into());
    }
    Ok(data)
}

pub fn get_tag_record_data_with_signed_authority_or_owner<'a>(
    program_id: &Pubkey,
    tag_record_info: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
) -> Result<TagRecordAccount, ProgramError> {
    if !owner.is_signer && !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<TagRecordAccount>(program_id, tag_record_info)?;

    if &data.authority != authority.key {
        return Err(TagError::InvalidAuthority.into());
    }

    if &data.owner != owner.key {
        return Err(TagError::InvalidOwner.into());
    }
    Ok(data)
}
