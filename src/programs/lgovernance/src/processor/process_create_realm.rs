//! Program state processor

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    error::GovernanceError,
    state::realm::{
        get_realm_mint_authority_program_address, get_realm_mint_program_address_seeds,
    },
    tokens::spl_utils::{
        create_spl_token_account_signed_with_bump,
    },
};

/// Processes create realm
pub fn process_create_realm(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    bump_seed: u8,
) -> ProgramResult {
    // For now just a spl token account
    let accounts_iter = &mut accounts.iter();
    let governing_token_holding_info = next_account_info(accounts_iter)?;
    let governing_token_transfer_authority_info = next_account_info(accounts_iter)?;
    let governing_token_mint_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let spl_token_info = next_account_info(accounts_iter)?;
    let rent_sysvar_info = next_account_info(accounts_iter)?;

    if governing_token_transfer_authority_info.key
        != &get_realm_mint_authority_program_address(program_id, governing_token_mint_info.key).0
    {
        return Err(GovernanceError::InvalidAuthorityForRealm.into());
    }

    if !governing_token_holding_info.data_is_empty() {
        return Err(GovernanceError::TokenHolderAccountAlreadyExist.into());
    }

    create_spl_token_account_signed_with_bump(
        governing_token_holding_info,
        &get_realm_mint_program_address_seeds(governing_token_mint_info.key, &[bump_seed]),
        governing_token_mint_info,
        governing_token_transfer_authority_info,
        payer_info,
        rent_sysvar_info,
        spl_token_info,
        system_info,
        program_id,
    )?;
    Ok(())
}
