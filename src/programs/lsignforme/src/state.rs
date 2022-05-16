use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{
    account::{get_account_data, MaxSize},
    content::ContentSource,
};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

use crate::error::SignForMeError;
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AccountType {
    SignerRecord,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct SignForMeAccount {
    pub account_type: AccountType,
    pub owner: Pubkey,
    pub signer: Pubkey,
    pub scope: Pubkey,
}

impl MaxSize for SignForMeAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for SignForMeAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::SignerRecord
    }
}

pub fn get_sign_for_me_data_for_owner_and_signer<'a>(
    program_id: &Pubkey,
    signer_for_me_info: &AccountInfo<'a>,
    owner: &Pubkey,
    signer: &AccountInfo<'a>,
) -> Result<SignForMeAccount, ProgramError> {
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<SignForMeAccount>(program_id, signer_for_me_info)?;

    if &data.owner != owner {
        return Err(SignForMeError::InvalidOwner.into());
    }

    if &data.signer != signer.key {
        return Err(SignForMeError::InvalidSigner.into());
    }
    Ok(data)
}

pub fn get_sign_for_me_data_for_signed_owner<'a>(
    program_id: &Pubkey,
    signer_for_me_info: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
) -> Result<SignForMeAccount, ProgramError> {
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<SignForMeAccount>(program_id, signer_for_me_info)?;
    if &data.owner != owner.key {
        return Err(SignForMeError::InvalidOwner.into());
    }

    Ok(data)
}
