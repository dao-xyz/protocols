use borsh::BorshSerialize;
use shared::content::ContentSource;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    create_user_account_program_address_seeds, shared::names::entity_name_is_valid,
    state::AccountType,
};

use super::{
    instruction::UserInstruction,
    state::{deserialize_user_account, UserAccount},
};
use shared::account::create_and_serialize_account_verify_with_bump;

pub struct Processor {}
impl Processor {
    // Program entrypoint's implementation

    pub fn process_create_user(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        name: String,
        profile: Option<ContentSource>,
        user_account_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        if !entity_name_is_valid(name.as_ref()) {
            return Err(ProgramError::InvalidArgument);
        }

        let user_acount_info = next_account_info(accounts_iter)?;
        if !user_acount_info.try_data_is_empty()? {
            // Already exist
            return Err(ProgramError::InvalidArgument);
        }
        let owner_info = next_account_info(accounts_iter)?;
        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let payer_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;
        let rent = Rent::get()?;
        let mut seeds = create_user_account_program_address_seeds(&name);
        seeds.push(vec![user_account_bump_seed]);

        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];

        let user_account = UserAccount {
            account_type: AccountType::User,
            name,
            profile,
            creation_timestamp: Clock::get()?.unix_timestamp as u64,
            owner: *owner_info.key, // payer becomes owner
        };

        create_and_serialize_account_verify_with_bump(
            payer_info,
            user_acount_info,
            &user_account,
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_update_user(
        accounts: &[AccountInfo],
        profile: Option<ContentSource>,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let user_account_info = next_account_info(accounts_iter)?;
        let owner_info = next_account_info(accounts_iter)?;
        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut user = deserialize_user_account(*user_account_info.data.borrow())?;
        if &user.owner != owner_info.key {
            return Err(ProgramError::InvalidAccountData);
        }
        user.profile = profile;
        user.serialize(&mut *user_account_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<UserInstruction>(data)?;
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
