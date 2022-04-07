//! Proposal Vote Record Account

use borsh::maybestd::io::Write;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{get_account_data, MaxSize};
use solana_program::account_info::AccountInfo;

use solana_program::program_error::ProgramError;
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

use crate::accounts::AccountType;
use crate::error::SocialError;

const VOTE_SEED: &[u8] = b"vote";

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum Vote {
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct VoteRecord {
    /// VoteRecord account type
    pub account_type: AccountType,

    /// Post
    pub post: Pubkey,

    /// Voter's vote
    pub vote: Vote,

    /// Owner of the vote record, and the authority for deletion
    pub owner: Pubkey,
}

impl MaxSize for VoteRecord {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 32 + 1 + 32)
    }
}

impl IsInitialized for VoteRecord {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::VoteRecord
    }
}

pub fn get_vote_record_data_for_signed_owner(
    program_id: &Pubkey,
    vote_record_info: &AccountInfo,
    owner: &AccountInfo,
) -> Result<VoteRecord, ProgramError> {
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let data = get_account_data::<VoteRecord>(program_id, vote_record_info)?;
    if &data.owner != owner.key {
        return Err(SocialError::InvalidOwnerForVoteRecord.into());
    }

    Ok(data)
}

pub fn get_vote_record_data_for_signed_owner_and_post(
    program_id: &Pubkey,
    vote_record_info: &AccountInfo,
    owner: &AccountInfo,
    post: &Pubkey,
) -> Result<VoteRecord, ProgramError> {
    let data = get_vote_record_data_for_signed_owner(program_id, vote_record_info, owner)?;
    if &data.post != post {
        return Err(SocialError::InvalidPostforVoteRecord.into());
    }
    Ok(data)
}

/// Returns VoteRecord PDA seeds
pub fn get_vote_record_address_seeds<'a>(
    post: &'a Pubkey,
    owner: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [VOTE_SEED, post.as_ref(), owner.as_ref(), bump_seed]
}

/// Returns VoteRecord PDA address
pub fn get_vote_record_address(program_id: &Pubkey, post: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VOTE_SEED, post.as_ref(), owner.as_ref()], program_id)
}
