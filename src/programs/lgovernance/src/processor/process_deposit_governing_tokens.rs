//! Program state processor

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::GovernanceError,
    state::{scopes::scope::VotePowerUnit, vote_power_origin_record::VotePowerOriginRecord},
    tokens::spl_utils::{get_spl_token_mint, get_spl_token_owner, transfer_spl_tokens},
};

/// Processes DepositGoverningTokens instruction
pub fn process_deposit_governing_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    token_origin_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governing_token_holding_info = next_account_info(accounts_iter)?;
    let governing_token_source_info = next_account_info(accounts_iter)?;
    let governing_owner_info = next_account_info(accounts_iter)?;
    let governing_token_transfer_authority_info = next_account_info(accounts_iter)?;
    let token_origin_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let spl_token_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    transfer_spl_tokens(
        governing_token_source_info,
        governing_token_holding_info,
        governing_token_transfer_authority_info,
        amount,
        spl_token_info,
    )?;

    let governing_owner = get_spl_token_owner(governing_token_source_info)?;
    let governing_token_mint = get_spl_token_mint(governing_token_source_info)?;

    if &governing_owner != governing_owner_info.key {
        return Err(GovernanceError::InvalidGoverningTokenHoldingAccount.into());
    }
    VotePowerOriginRecord::create(
        program_id,
        VotePowerUnit::Mint(governing_token_mint),
        amount,
        &rent,
        token_origin_record_info,
        token_origin_record_bump_seed,
        governing_owner_info,
        payer_info,
        system_info,
    )?;
    Ok(())
}
