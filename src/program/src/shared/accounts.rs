/// Check account owner is the given program
pub fn check_account_owner(
    account_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if *program_id != *account_info.owner {
        msg!(
            "Expected account to be owned by program {}, received {}",
            program_id,
            account_info.owner
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}