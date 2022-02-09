use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    shared::names::entity_name_is_valid,
    socials::{
        create_and_serialize_account_signed_verify, create_user_account_program_address_seeds,
        state::AccountContainer,
    },
};

use super::{
    instruction::UserInstruction,
    state::{deserialize_user_account, UserAccount},
};

pub struct Processor {}
impl Processor {
    // Program entrypoint's implementation

    pub fn process_create_user(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        name: String,
        profile: Option<String>,
        user_account_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;

        if !entity_name_is_valid(name.as_ref()) {
            return Err(ProgramError::InvalidArgument);
        }

        let user_acount_info = next_account_info(accounts_iter)?;
        if !user_acount_info.try_data_is_empty()? {
            // Already exist
            return Err(ProgramError::InvalidArgument);
        }
        let system_account = next_account_info(accounts_iter)?;
        let rent = Rent::get()?;
        let mut seeds = create_user_account_program_address_seeds(&name);
        seeds.push(vec![user_account_bump_seed]);

        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];

        let user_account = UserAccount {
            name,
            profile,
            creation_timestamp: Clock::get()?.unix_timestamp as u64,
            owner: *payer_account.key, // payer becomes owner
        };

        create_and_serialize_account_signed_verify(
            payer_account,
            user_acount_info,
            &AccountContainer::UserAccount(user_account),
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_update_user(accounts: &[AccountInfo], profile: Option<String>) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;

        if !payer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let user_account_info = next_account_info(accounts_iter)?;
        let mut user = deserialize_user_account(*user_account_info.data.borrow())?;
        if &user.owner != payer_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        user.profile = profile;
        AccountContainer::UserAccount(user).serialize(&mut *user_account_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: UserInstruction,
    ) -> ProgramResult {
        match instruction {
            UserInstruction::CreateUser {
                name,
                profile,

                user_account_bump_seed,
            } => {
                msg!("Instruction: Create user");
                Self::process_create_user(
                    program_id,
                    accounts,
                    name,
                    profile,
                    user_account_bump_seed,
                )
            }
            UserInstruction::UpdateUser { profile } => {
                msg!("Instruction: Update user");
                Self::process_update_user(accounts, profile)
            }
        }
    }
}
