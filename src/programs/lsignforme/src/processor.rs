use borsh::BorshSerialize;
use shared::{
    account::{dispose_account, get_account_data},
    content::ContentSource,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use shared::account::create_and_serialize_account_verify_with_bump;

use crate::{
    error::SignForMeError,
    get_sign_for_me_program_address_seeds,
    instruction::SignForMeInstruction,
    state::{get_sign_for_me_data_for_signed_owner, AccountType, SignForMeAccount},
};

pub struct Processor {}
impl Processor {
    pub fn process_create_sign_for_me(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        signer: Pubkey,
        scope: Pubkey,
        bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let sign_for_me_info = next_account_info(accounts_iter)?;
        let owner_info = next_account_info(accounts_iter)?;
        let payer_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        if !sign_for_me_info.try_data_is_empty()? {
            // Already exist
            return Err(SignForMeError::SignForMeRecordAlreadyExist.into());
        }

        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let rent = Rent::get()?;
        let bump_seeds = [bump_seed];
        let seeds =
            get_sign_for_me_program_address_seeds(owner_info.key, &signer, &scope, &bump_seeds);

        create_and_serialize_account_verify_with_bump(
            payer_info,
            sign_for_me_info,
            &SignForMeAccount {
                account_type: AccountType::SignerRecord,
                owner: *owner_info.key,
                scope,
                signer,
            },
            &seeds,
            program_id,
            system_account,
            &rent,
        )?;

        Ok(())
    }

    pub fn process_delete_sign_for_me_as_owner(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let sign_for_me_info = next_account_info(accounts_iter)?;
        let owner_info = next_account_info(accounts_iter)?;
        let destination_account_info = next_account_info(accounts_iter)?;
        let _sign_for_me_data =
            get_sign_for_me_data_for_signed_owner(program_id, sign_for_me_info, owner_info)?;
        dispose_account(sign_for_me_info, destination_account_info);
        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<SignForMeInstruction>(data)?;
        match instruction {
            SignForMeInstruction::CreateSignForMe {
                bump_seed,
                scope,
                signer,
            } => {
                msg!("Instruction: CreateSignForMe");
                Self::process_create_sign_for_me(program_id, accounts, signer, scope, bump_seed)
            }
            SignForMeInstruction::DeleteSignForMe => {
                msg!("Instruction: DeleteSignForMe");
                Self::process_delete_sign_for_me_as_owner(program_id, accounts)
            }
        }
    }
}
