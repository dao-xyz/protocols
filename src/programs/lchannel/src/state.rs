use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{account::MaxSize, content::ContentSource};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, program_error::ProgramError,
    pubkey::Pubkey,
};

pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;

pub const MAX_CHANNEL_LEN: usize = 1 + 32 // owner pubkey
    + 8 // timestamp
    + MAX_NAME_LENGTH
    + 1  // option
    + MAX_URI_LENGTH
    + 200; // some padding

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum AccountType {
    Channel,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ChannelAccount {
    pub account_type: AccountType,
    pub creator: Pubkey,
    pub creation_timestamp: u64,
    pub parent: Option<Pubkey>,
    pub name: String,
    pub link: Option<ContentSource>, // The link to to info data

    // Should be set to itself make it self governed through proposals
    // as it can only sign itself through the program (program functions)
    pub authority: Pubkey,
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_CHANNEL_LEN)
    }
}

impl ChannelAccount {
    pub fn check_authority<'a>(
        &self,
        authority_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if &self.authority != authority_info.key {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub fn deserialize_channel_account(data: &[u8]) -> std::io::Result<ChannelAccount> {
    let account: ChannelAccount = try_from_slice_unchecked(data)?;
    Ok(account)
}
