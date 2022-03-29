use crate::state::{proposal::OptionVoteResult, token_owner_record::TokenOwnerRecordV2};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_delegatee(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    rule: Pubkey,
    governing_token_mint: Pubkey,
    token_owner_record_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governing_token_owner_info = next_account_info(accounts_iter)?;
    let token_owner_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    TokenOwnerRecordV2::create_empty_delegate(
        program_id,
        &rule,
        &rent,
        token_owner_record_info,
        token_owner_record_bump_seed,
        governing_token_owner_info,
        &governing_token_mint,
        payer_info,
        system_info,
    )?;
    Ok(())
}
