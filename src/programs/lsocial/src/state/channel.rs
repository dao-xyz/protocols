use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use shared::seeds::generate_seeds_from_string;
use shared::{account::MaxSize, content::ContentSource};

use solana_program::clock::UnixTimestamp;
use solana_program::{
    program_pack::IsInitialized,
    pubkey::{Pubkey, PubkeyError},
};

use crate::accounts::AccountType;

pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 100;
const CHANNEL_SEED: &[u8] = b"channel";

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum ChannelType {
    Collection,
    PostStream,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ChannelAccount {
    pub account_type: AccountType,
    pub channel_type: ChannelType,
    pub creation_timestamp: UnixTimestamp,
    pub parent: Option<Pubkey>,
    pub name: String,
    pub link: Option<ContentSource>,
    pub collection: Option<Pubkey>,
    pub encryption: Encryption,
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        return Some(1 + 1 + 8 + MAX_NAME_LENGTH + 33 + 1 + MAX_URI_LENGTH + 33 + 2 + 100);
        // Last is padding
    }
}

impl IsInitialized for ChannelAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Channel
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum Encryption {
    None,
}

pub fn get_channel_program_address(
    program_id: &Pubkey,
    channel_name: &str,
    parent: Option<&Pubkey>,
) -> Result<(Pubkey, u8), PubkeyError> {
    let mut seed = generate_seeds_from_string(channel_name)?;
    seed.insert(0, CHANNEL_SEED.to_vec());
    if let Some(parent) = parent {
        seed.push(parent.as_ref().to_vec());
    }
    let seed_slice = &seed.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
    Ok(Pubkey::find_program_address(seed_slice, program_id))
}

pub fn get_channel_account_program_address_seeds(
    channel_name: &str,
    parent: Option<&Pubkey>,
) -> Result<Vec<Vec<u8>>, PubkeyError> {
    let mut seed = generate_seeds_from_string(channel_name)?;
    seed.insert(0, CHANNEL_SEED.to_vec());
    if let Some(parent) = parent {
        seed.push(parent.as_ref().to_vec());
    }
    Ok(seed)
}
