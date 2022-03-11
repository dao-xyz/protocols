use std::slice::Iter;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::seeds::generate_seeds_from_string;
use shared::{account::MaxSize, content::ContentSource, error::UtilsError};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::{Pubkey, PubkeyError},
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
pub enum ChannelAuthority {
    AuthorityByTag { tag: Pubkey, authority: Pubkey },
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ChannelAccount {
    pub account_type: AccountType,
    pub creator: Pubkey,
    pub creation_timestamp: u64,
    pub parent: Option<Pubkey>,
    pub name: String,
    pub link: Option<ContentSource>, // The link to to info data
    /*
    // A key controlling its governance of something,
    pub governance: Pubkey,
     */
    // Should be set to itself make it self governed through proposals
    // as it can only sign itself through the program (program functions)
    pub authority: Pubkey,

    // Tag that lets users create posts
    pub channel_authority_config: ChannelAuthority,
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(MAX_CHANNEL_LEN)
    }
}

impl IsInitialized for ChannelAccount {
    fn is_initialized(&self) -> bool {
        return self.account_type == AccountType::Channel;
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

    pub fn find_channel_program_address(
        &self,
        program_id: &Pubkey,
    ) -> Result<(Pubkey, u8), PubkeyError> {
        let seeds = create_channel_account_program_address_seeds(self.name.as_str())?;
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
        Ok(Pubkey::find_program_address(seed_slice, program_id))
    }

    pub fn create_channel_account_program_address_seeds(
        &self,
    ) -> Result<Vec<Vec<u8>>, PubkeyError> {
        generate_seeds_from_string(self.name.as_str())
    }
}

/// Findchannel address from name
pub fn find_channel_program_address(
    program_id: &Pubkey,
    channel_name: &str,
) -> Result<(Pubkey, u8), PubkeyError> {
    let seeds = create_channel_account_program_address_seeds(channel_name)?;
    let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Ok(Pubkey::find_program_address(seed_slice, program_id))
}

/// Create post mint program address
pub fn create_channel_account_program_address_seeds(
    channel_name: &str,
) -> Result<Vec<Vec<u8>>, PubkeyError> {
    generate_seeds_from_string(channel_name)
}
