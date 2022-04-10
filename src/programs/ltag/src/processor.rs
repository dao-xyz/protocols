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
    error::TagError,
    get_tag_program_address_seeds, get_tag_record_factory_program_address_seeds,
    get_tag_record_program_address_seeds,
    instruction::TagInstruction,
    names::entity_name_is_valid,
    state::{
        get_tag_record_data_with_authority, get_tag_record_factory_with_authority, AccountType,
        TagAccount, TagRecordAccount, TagRecordFactoryAccount,
    },
};

pub struct Processor {}
impl Processor {
    pub fn process_create_tag(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        tag: String,
        info: Option<ContentSource>,
        tag_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let tag_info = next_account_info(accounts_iter)?;
        let authority_info = next_account_info(accounts_iter)?;

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

        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let rent = Rent::get()?;
        let mut seeds = get_tag_program_address_seeds(&tag);
        seeds.push(vec![tag_bump_seed]);
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];

        let tag_account = TagAccount {
            account_type: AccountType::Tag,
            tag,
            authority: *authority_info.key,
            info,
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
        let tag_record_owner_info = next_account_info(accounts_iter)?;

        let tag_record_factory_info = next_account_info(accounts_iter)?;
        let tag_record_authority_info = next_account_info(accounts_iter)?;

        let payer_account = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        let mut factory_data = get_tag_record_factory_with_authority(
            program_id,
            tag_record_factory_info,
            tag_record_authority_info,
        )?;

        factory_data.outstanding_records = factory_data.outstanding_records.checked_add(1).unwrap();

        if !tag_record_info.try_data_is_empty()? {
            // Expected not to exist
            return Err(TagError::TagRecordAlreadyExist.into());
        }

        let rent = Rent::get()?;
        let bump_seeds = [tag_record_bump_seed];
        let seeds = get_tag_record_program_address_seeds(
            tag_record_factory_info.key,
            tag_record_owner_info.key,
            &bump_seeds,
        );

        let tag_account = TagRecordAccount {
            account_type: AccountType::TagRecord,
            factory: *tag_record_factory_info.key,
            owner: *tag_record_owner_info.key,
            tag: factory_data.tag,
        };

        factory_data.serialize(&mut *tag_record_factory_info.data.borrow_mut())?;
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
        let tag_record_owner_info = next_account_info(accounts_iter)?;

        let tag_record_factory_info = next_account_info(accounts_iter)?;
        let tag_record_authority_info = next_account_info(accounts_iter)?;

        let destination_account_info = next_account_info(accounts_iter)?;

        // Verify authority
        let mut factory_data =
            get_account_data::<TagRecordFactoryAccount>(program_id, tag_record_factory_info)?;

        if &factory_data.authority != tag_record_authority_info.key {
            return Err(TagError::InvalidAuthority.into());
        }

        factory_data.outstanding_records = factory_data.outstanding_records.checked_sub(1).unwrap();

        let _tag_record = get_tag_record_data_with_authority(
            program_id,
            tag_record_info,
            tag_record_factory_info,
            &factory_data,
            tag_record_authority_info,
            tag_record_owner_info,
        )?;

        factory_data.serialize(&mut *tag_record_factory_info.data.borrow_mut())?;
        dispose_account(tag_record_info, destination_account_info);

        Ok(())
    }

    pub fn process_create_tag_record_factory(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        tag_record_factory_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let tag_record_factory_info = next_account_info(accounts_iter)?;
        let tag_authority_info = next_account_info(accounts_iter)?;
        let tag_info = next_account_info(accounts_iter)?;

        let payer_account = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        if !tag_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !tag_record_factory_info.try_data_is_empty()? {
            // Expected not to exist
            return Err(TagError::TagRecordFactoryAlreadyExist.into());
        }

        if tag_info.try_data_is_empty()? {
            // Expected to exist
            return Err(TagError::TagDoesNotExist.into());
        }

        let rent = Rent::get()?;
        let bump_seeds = [tag_record_factory_bump_seed];
        let seeds = get_tag_record_factory_program_address_seeds(
            tag_info.key,
            tag_authority_info.key,
            &bump_seeds,
        );

        let tag_record_factory_account = TagRecordFactoryAccount {
            account_type: AccountType::TagRecordFactory,
            authority: *tag_authority_info.key,
            tag: *tag_info.key,
            outstanding_records: 0,
            authority_can_withdraw: false,
            owner_can_transfer: false,
        };

        create_and_serialize_account_verify_with_bump(
            payer_account,
            tag_record_factory_info,
            &tag_record_factory_account,
            &seeds,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<TagInstruction>(data)?;
        match instruction {
            TagInstruction::CreateTag {
                bump_seed,
                tag,
                info,
            } => {
                msg!("Instruction: Create tag");
                Self::process_create_tag(program_id, accounts, tag, info, bump_seed)
            }
            TagInstruction::CreateTagRecord { bump_seed } => {
                msg!("Instruction: Create tag record");
                Self::process_create_tag_record(program_id, accounts, bump_seed)
            }
            TagInstruction::DeleteTagRecord => {
                msg!("Instruction: Delete tag record");
                Self::process_delete_tag_record(program_id, accounts)
            }
            TagInstruction::CreateTagRecordFactory { tag: _, bump_seed } => {
                msg!("Instruction: Create tag record factory");
                Self::process_create_tag_record_factory(program_id, accounts, bump_seed)
            }
        }
    }
}
