//! Program state processor

use crate::{error::GovernanceError, state::governance::GovernanceV2};
use borsh::BorshSerialize;
use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes CreateGovernance instruction
pub fn process_update_governance_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_authority: Option<Pubkey>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let governance_info = next_account_info(account_info_iter)?;
    let mut governance_data = get_account_data::<GovernanceV2>(program_id, governance_info)?;
    if !governance_info.is_signer {
        if let Some(authority) = &governance_data.optional_authority {
            let governance_authority = next_account_info(account_info_iter)?;
            if authority != governance_authority.key {
                return Err(GovernanceError::InvalidAuthorityForGovernance.into());
            }
            if !governance_authority.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
        } else {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }

    governance_data.optional_authority = new_authority;
    governance_data.serialize(&mut *governance_info.data.borrow_mut())?;
    Ok(())
}
