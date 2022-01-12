use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::shared::account::{get_token_account_mint, get_token_account_owner, get_token_balance};

pub mod program_owner_token {

    solana_program::declare_id!("52oKpSaBQCP422hfDXvGAPZXVwhGbN6ZwkimgyJFcWAw");
}

pub fn assert_is_signing_program_owner<'a>(
    owner_account: &AccountInfo<'a>,
    owner_token_account: &AccountInfo<'a>,
) -> ProgramResult {
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // assert balance
    let owner = get_token_account_owner(&owner_token_account)?;
    if &owner != owner_account.key {
        msg!(
            "Token account owner does not match. Found {:?} but {:?} was expected",
            owner_account.key,
            owner
        );
        return Err(ProgramError::IllegalOwner);
    }

    let mint = get_token_account_mint(&owner_token_account)?;

    if !program_owner_token::check_id(&mint) {
        msg!(
            "Wrong owner token mint. Found {:?} but {:?} was expected",
            mint,
            program_owner_token::id()
        );
        return Err(ProgramError::IllegalOwner);
    }

    let balance = get_token_balance(&owner_token_account)?;
    if balance < 1 {
        msg!("Token account holds insufficient goverence balance");
        return Err(ProgramError::IllegalOwner);
    }

    Ok(())
}
