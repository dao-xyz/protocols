//! Program state processor

use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::governance::{get_governance_address_seeds, GovernanceV2},
};
use lchannel::state::ChannelAccount;
use shared::account::{
    create_and_serialize_account_signed, create_and_serialize_account_verify_with_bump,
    get_account_data,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Processes CreateGovernance instruction
pub fn process_create_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    bump_seed: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let governance_info = next_account_info(account_info_iter)?;
    let channel_info = next_account_info(account_info_iter)?;
    let channel_info_authority = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;

    let rent = Rent::get()?;

    let channel_data = get_account_data::<ChannelAccount>(&lchannel::id(), channel_info)?;
    if &channel_data.authority != channel_info_authority.key {
        return Err(GovernanceError::InvalidAuthorityForChannel.into());
    }

    if !channel_info_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let governance_data = GovernanceV2 {
        account_type: AccountType::Governance,
        channel: *channel_info.key,
        proposals_count: 0,
        voting_proposal_count: 0,
    };

    create_and_serialize_account_verify_with_bump::<GovernanceV2>(
        payer_info,
        governance_info,
        &governance_data,
        &get_governance_address_seeds(channel_info.key, &[bump_seed]),
        program_id,
        system_info,
        &rent,
    )?;

    Ok(())
}
