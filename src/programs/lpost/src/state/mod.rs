use ltag::state::get_tag_record_data_with_signed_authority_or_owner;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::error::PostError;

pub mod post;
pub mod vote_record;

pub fn assert_authorized_by_tag<'a>(
    tag_owner_info: &AccountInfo<'a>, //
    tag_record_info: &AccountInfo<'a>,
    expected_tag: &Pubkey,
    expected_tag_authority: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    if !tag_owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let tag_record_data = get_tag_record_data_with_signed_authority_or_owner(
        &ltag::id(),
        tag_record_info,
        expected_tag_authority,
        tag_owner_info,
    )?;

    if &tag_record_data.tag != expected_tag {
        return Err(PostError::InvalidTagForVote.into());
    }

    Ok(())
}
