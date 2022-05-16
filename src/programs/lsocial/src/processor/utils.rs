use std::slice::Iter;

use lsignforme::state::get_sign_for_me_data_for_owner_and_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    msg,
    program_error::ProgramError,
};

pub fn verify_signed_owner_maybe_sign_for_me(
    owner_info: &AccountInfo,
    accounts_iter: &mut Iter<AccountInfo>,
) -> Result<(), ProgramError> {
    // Assume sign for me
    if !owner_info.is_signer {
        let sign_for_me_info = next_account_info(accounts_iter)?;
        let sign_for_me_signer = next_account_info(accounts_iter)?;
        let _sign_for_me_data = get_sign_for_me_data_for_owner_and_signer(
            &lsignforme::id(),
            sign_for_me_info,
            owner_info.key,
            sign_for_me_signer,
        )?;
    }
    Ok(())
}
