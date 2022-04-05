//! Program state processor

use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        token_owner_budget_record::{
            get_token_owner_budget_record_address_seeds, TokenOwnerBudgetRecord,
        },
        vote_power_origin_record::VotePowerOriginRecord,
    },
};
use shared::account::{create_and_serialize_account_verify_with_bump, get_account_data};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_token_owner_budget_record(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    scope: Pubkey,
    token_owner_budget_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_origin_record_info = next_account_info(accounts_iter)?;
    let token_owner_budget_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    let token_origin_record =
        get_account_data::<VotePowerOriginRecord>(program_id, token_origin_record_info)?;

    let token_owner_budget_record_bump_seeds = [token_owner_budget_record_bump_seed];
    let seeds = get_token_owner_budget_record_address_seeds(
        token_origin_record_info.key,
        &scope,
        &token_owner_budget_record_bump_seeds,
    );

    if token_owner_budget_record_info.data_is_empty() {
        create_and_serialize_account_verify_with_bump::<TokenOwnerBudgetRecord>(
            payer_info,
            token_owner_budget_record_info,
            &TokenOwnerBudgetRecord {
                account_type: AccountType::TokenOwnerBudgetRecord,
                /*
                amount: match token_owner_record.source {
                    VoteSource::Mint(governing_token_deposit_amount) => {
                        governing_token_deposit_amount
                    }
                    VoteSource::Tag { .. } => 1, // Binary, 1 if not delegated, else 0
                }, */
                scope,
                amount: token_origin_record.amount,
                token_owner_record: *token_origin_record_info.key,
            },
            &seeds,
            program_id,
            system_info,
            &rent,
        )?;
    } else {
        return Err(GovernanceError::TokenOwnerBudgetRecordMissing.into());
    }

    Ok(())
}
