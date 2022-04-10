use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{
    account::{get_account_data, MaxSize},
    content::ContentSource,
};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::error::TagError;
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AccountType {
    Tag,
    TagRecord,
    TagRecordFactory,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TagAccount {
    pub account_type: AccountType,
    pub tag: String,
    pub info: Option<ContentSource>,
    pub authority: Pubkey,
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
    pub factory: Pubkey,
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
pub fn get_tag_record_data_with_factory_and_signed_owner<'a>(
    program_id: &Pubkey,
    tag_record_info: &AccountInfo<'a>,
    factory: &Pubkey,
    owner: &AccountInfo<'a>,
) -> Result<TagRecordAccount, ProgramError> {
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<TagRecordAccount>(program_id, tag_record_info)?;

    if &data.factory != factory {
        return Err(TagError::InvalidTagRecordFactory.into());
    }

    if &data.owner != owner.key {
        return Err(TagError::InvalidOwner.into());
    }
    Ok(data)
}

pub fn get_tag_record_data_with_authority<'a>(
    program_id: &Pubkey,
    tag_record_info: &AccountInfo<'a>,
    factory_info: &AccountInfo<'a>,
    factory_data: &TagRecordFactoryAccount,
    factory_authority_info: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
) -> Result<TagRecordAccount, ProgramError> {
    let data = get_account_data::<TagRecordAccount>(program_id, tag_record_info)?;
    if owner.is_signer {
        if &data.owner != owner.key {
            return Err(TagError::InvalidOwner.into());
        }
        Ok(data)
    } else if factory_authority_info.is_signer {
        if &factory_data.authority != factory_authority_info.key {
            return Err(TagError::InvalidAuthority.into());
        }
        if &data.factory != factory_info.key {
            return Err(TagError::InvalidTagRecordFactory.into());
        }
        Ok(data)
    } else {
        Err(ProgramError::MissingRequiredSignature)
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TagRecordFactoryAccount {
    pub account_type: AccountType,
    pub tag: Pubkey,
    pub authority: Pubkey,
    pub outstanding_records: u64,
    pub owner_can_transfer: bool,
    pub authority_can_withdraw: bool,
}

impl MaxSize for TagRecordFactoryAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 32 + 32 + 8 + 1 + 1)
    }
}

impl IsInitialized for TagRecordFactoryAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::TagRecordFactory
    }
}

pub fn get_tag_record_factory_with_authority<'a>(
    program_id: &Pubkey,
    tag_record_factory_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
) -> Result<TagRecordFactoryAccount, ProgramError> {
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<TagRecordFactoryAccount>(program_id, tag_record_factory_info)?;

    if &data.authority != authority_info.key {
        return Err(TagError::InvalidAuthority.into());
    }

    Ok(data)
}
