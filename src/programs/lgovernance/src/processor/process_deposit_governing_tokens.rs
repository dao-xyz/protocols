//! Program state processor

use shared::account::create_and_serialize_account_signed;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::token_owner_record::{
        get_token_owner_record_address_seeds, get_token_owner_record_data_for_seeds,
        TokenOwnerRecordV2,
    },
    tokens::spl_utils::{
        create_spl_token_account_signed, get_spl_token_mint, get_spl_token_owner,
        transfer_spl_tokens,
    },
};

/// Processes DepositGoverningTokens instruction
pub fn process_deposit_governing_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    token_owner_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governing_token_holding_info = next_account_info(accounts_iter)?;
    let governing_token_source_info = next_account_info(accounts_iter)?;
    let governing_token_owner_info = next_account_info(accounts_iter)?;
    let governing_token_transfer_authority_info = next_account_info(accounts_iter)?;
    let token_owner_record_info = next_account_info(accounts_iter)?;
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

    let governing_token_owner = get_spl_token_owner(governing_token_source_info)?;
    let governing_token_mint = get_spl_token_mint(governing_token_source_info)?;
    TokenOwnerRecordV2::create(
        program_id,
        amount,
        &rent,
        token_owner_record_info,
        token_owner_record_bump_seed,
        &governing_token_owner,
        governing_token_owner_info,
        &governing_token_mint,
        payer_info,
        system_info,
    )?;
    Ok(())
}

/* let account_info_iter = &mut accounts.iter();

let governing_token_holding_info = next_account_info(account_info_iter)?;
let governing_token_source_info = next_account_info(account_info_iter)?;
let governing_token_owner_info = next_account_info(account_info_iter)?;
let governing_token_transfer_authority_info = next_account_info(account_info_iter)?;
let token_owner_record_info = next_account_info(account_info_iter)?;
let payer_info = next_account_info(account_info_iter)?;
let system_info = next_account_info(account_info_iter)?;
let spl_token_info = next_account_info(account_info_iter)?; // 8

let rent = Rent::get()?;

let realm_data = get_realm_data(program_id, realm_info)?;
let governing_token_mint = get_spl_token_mint(governing_token_holding_info)?;

realm_data.asset_governing_tokens_deposits_allowed(&governing_token_mint)?;

realm_data.assert_is_valid_governing_token_mint_and_holding(
    program_id,
    realm_info.key,
    &governing_token_mint,
    governing_token_holding_info.key,
)?;

transfer_spl_tokens(
    governing_token_source_info,
    governing_token_holding_info,
    governing_token_transfer_authority_info,
    amount,
    spl_token_info,
)?;

let token_owner_record_address_seeds = get_token_owner_record_address_seeds(
    realm_info.key,
    &governing_token_mint,
    governing_token_owner_info.key,
);

if token_owner_record_info.data_is_empty() {
    // Deposited tokens can only be withdrawn by the owner so let's make sure the owner signed the transaction
    let governing_token_owner = get_spl_token_owner(governing_token_source_info)?;

    if !(governing_token_owner == *governing_token_owner_info.key
        && governing_token_owner_info.is_signer)
    {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }

    let token_owner_record_data = TokenOwnerRecordV2 {
        account_type: GovernanceAccountType::TokenOwnerRecordV2,
        realm: *realm_info.key,
        governing_token_owner: *governing_token_owner_info.key,
        governing_token_deposit_amount: amount,
        governing_token_mint,
        governance_delegate: None,
        unrelinquished_votes_count: 0,
        total_votes_count: 0,
        outstanding_proposal_count: 0,
        reserved: [0; 7],
        reserved_v2: [0; 128],
    };

    create_and_serialize_account_signed(
        payer_info,
        token_owner_record_info,
        &token_owner_record_data,
        &token_owner_record_address_seeds,
        program_id,
        system_info,
        &rent,
    )?;
} else {
    let mut token_owner_record_data = get_token_owner_record_data_for_seeds(
        program_id,
        token_owner_record_info,
        &token_owner_record_address_seeds,
    )?;

    token_owner_record_data.governing_token_deposit_amount = token_owner_record_data
        .governing_token_deposit_amount
        .checked_add(amount)
        .unwrap();

    token_owner_record_data.serialize(&mut *token_owner_record_info.data.borrow_mut())?;
} */
