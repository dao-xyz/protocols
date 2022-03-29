use shared::account::{dispose_account, MaxSize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::sol_memset,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use shared::account::create_and_serialize_account_verify_with_bump;

use crate::{
    error::TagError,
    get_tag_program_address_seeds, get_tag_record_program_address_seeds,
    instruction::TagInstruction,
    names::entity_name_is_valid,
    state::{
        get_tag_record_data_with_authority_and_signed_owner,
        get_tag_record_data_with_signed_authority_or_owner, AccountType, TagAccount,
        TagRecordAccount,
    },
};

pub struct Processor {}
impl Processor {
    pub fn process_create_tag(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        tag: String,
        tag_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let tag_info = next_account_info(accounts_iter)?;
        /*         let tag_owner_info = next_account_info(accounts_iter)?;
        let tag_authority_info = next_account_info(accounts_iter)?; */
        let payer_account = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        if !entity_name_is_valid(tag.as_ref()) {
            return Err(ProgramError::InvalidArgument);
        }
        /*
        if !tag_authority_info.is_signer
        {
            return Err(ProgramError::MissingRequiredSignature)
        } */

        if !tag_info.try_data_is_empty()? {
            // Already exist
            return Err(TagError::TagAlreadyExist.into());
        }
        let rent = Rent::get()?;
        let mut seeds = get_tag_program_address_seeds(&tag);
        seeds.push(vec![tag_bump_seed]);
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];

        let tag_account = TagAccount {
            account_type: AccountType::Tag,
            tag,
        };

        create_and_serialize_account_verify_with_bump(
            payer_account,
            tag_info,
            &tag_account,
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_create_tag_record(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        tag_record_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let tag_record_info = next_account_info(accounts_iter)?;
        let tag_info = next_account_info(accounts_iter)?;
        let tag_authority_info = next_account_info(accounts_iter)?;
        let tag_owner_info = next_account_info(accounts_iter)?;

        let payer_account = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        if !tag_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !tag_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if tag_info.try_data_is_empty()? {
            // Expected to exist
            return Err(TagError::TagDoesNotExist.into());
        }

        if !tag_record_info.try_data_is_empty()? {
            // Expected not to exist
            return Err(TagError::TagRecordAlreadyExist.into());
        }

        let rent = Rent::get()?;
        let bump_seeds = [tag_record_bump_seed];
        let seeds = get_tag_record_program_address_seeds(
            tag_info.key,
            tag_owner_info.key,
            tag_authority_info.key,
            &bump_seeds,
        );

        let tag_account = TagRecordAccount {
            account_type: AccountType::TagRecord,
            authority: *tag_authority_info.key,
            owner: *tag_owner_info.key,
            tag: *tag_info.key,
        };

        create_and_serialize_account_verify_with_bump(
            payer_account,
            tag_record_info,
            &tag_account,
            &seeds,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_delete_tag_record(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let tag_record_info = next_account_info(accounts_iter)?;
        let tag_authority_info = next_account_info(accounts_iter)?;
        let tag_owner_info = next_account_info(accounts_iter)?;
        let destination_account_info = next_account_info(accounts_iter)?;

        // Verify authority
        let _tag_record = get_tag_record_data_with_signed_authority_or_owner(
            program_id,
            tag_record_info,
            tag_authority_info,
            tag_owner_info,
        )?;

        dispose_account(tag_record_info, destination_account_info);

        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<TagInstruction>(data)?;
        match instruction {
            TagInstruction::CreateTag { bump_seed, tag } => {
                msg!("Instruction: Create tag");
                Self::process_create_tag(program_id, accounts, tag, bump_seed)
            }
            TagInstruction::CreateTagRecord { bump_seed } => {
                msg!("Instruction: Create tag record");
                Self::process_create_tag_record(program_id, accounts, bump_seed)
            }
            TagInstruction::DeleteTagRecord => {
                msg!("Instruction: Delete tag record");
                Self::process_delete_tag_record(program_id, accounts)
            }
        }
    }
}
