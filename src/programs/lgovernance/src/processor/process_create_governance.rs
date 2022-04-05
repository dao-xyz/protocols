//! Program state processor

use crate::{
    accounts::AccountType,
    state::governance::{get_governance_address_seeds, GovernanceV2},
};
use shared::account::create_and_serialize_account_verify_with_bump;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Processes CreateGovernance instruction
pub fn process_create_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_authority: Pubkey,
    seed: Pubkey,
    bump_seed: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let governance_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;

    let rent = Rent::get()?;

    let governance_data = GovernanceV2 {
        account_type: AccountType::Governance,
        optional_authority: Some(initial_authority),
        proposals_count: 0,
        voting_proposal_count: 0,
        seed,
    };

    create_and_serialize_account_verify_with_bump::<GovernanceV2>(
        payer_info,
        governance_info,
        &governance_data,
        &get_governance_address_seeds(&seed, &[bump_seed]),
        program_id,
        system_info,
        &rent,
    )?;

    Ok(())
}
