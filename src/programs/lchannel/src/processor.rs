use borsh::BorshSerialize;
use shared::{
    account::{
        check_account_owner, create_and_serialize_account_verify_with_bump, get_account_data,
    },
    content::ContentSource,
};
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
    shared::names::entity_name_is_valid,
    state::{create_channel_account_program_address_seeds, AccountType, ChannelAuthority},
};

use super::{instruction::ChannelInstruction, state::ChannelAccount};

pub struct Processor {}
impl Processor {
    // Program entrypoint's implementation

    pub fn process_create_channel(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        parent: Option<Pubkey>,
        name: String,
        link: Option<ContentSource>,
        channel_authority_config: ChannelAuthority,
        channel_account_bump_seed: u8, /*
                                       create_rule_address_bump_seed: u8, */
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let channel_account_info = next_account_info(accounts_iter)?;
        let creator = next_account_info(accounts_iter)?;
        let payer_account = next_account_info(accounts_iter)?;

        if !creator.is_signer {
            // Do not let someone create an channel for someone else without their signature
            return Err(ProgramError::MissingRequiredSignature);
        }
        if !entity_name_is_valid(name.as_ref()) {
            return Err(ProgramError::InvalidArgument);
        }

        if !channel_account_info.try_data_is_empty()? {
            // Channel already exist
            return Err(ProgramError::InvalidAccountData);
        }

        let system_account = next_account_info(accounts_iter)?;

        if parent.is_some() {
            let parent_channel_account_info = next_account_info(accounts_iter)?;
            let parent_channel =
                get_account_data::<ChannelAccount>(program_id, parent_channel_account_info)?;
            let parent_channel_authority_info = next_account_info(accounts_iter)?;

            if &parent_channel.authority != parent_channel_authority_info.key {
                return Err(ProgramError::InvalidAccountData);
            }

            if !parent_channel_authority_info.is_signer {
                msg!("Not signer");
                return Err(ProgramError::MissingRequiredSignature); // Is signed, means the program has signed it, since its a PDA
            }
        }
        let rent = Rent::get()?;
        /*
           Channel and user names must be unique, as we generate the seeds in the same way for both
           Do we want this really?
        */
        let mut seeds = create_channel_account_program_address_seeds(name.as_ref())?;
        seeds.push(vec![channel_account_bump_seed]);
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
        create_and_serialize_account_verify_with_bump(
            payer_account,
            channel_account_info,
            &ChannelAccount {
                account_type: AccountType::Channel,
                creator: *creator.key,
                parent,
                link,
                name,
                creation_timestamp: Clock::get()?.unix_timestamp as u64,
                authority: *payer_account.key,
                channel_authority_config,
            },
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;

        Ok(())
    }

    // This should actually be voted on..
    pub fn process_update_authority(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        new_authority: Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let channel_account_info = next_account_info(accounts_iter)?;
        if channel_account_info.try_data_is_empty()? {
            // Channel does not exist
            return Err(ProgramError::InvalidArgument);
        }
        let mut channel = get_account_data::<ChannelAccount>(program_id, channel_account_info)?;
        let authority_account = next_account_info(accounts_iter)?;

        channel.check_authority(authority_account)?;

        channel.authority = new_authority;
        channel.serialize(&mut *channel_account_info.data.borrow_mut())?;
        Ok(())
    }

    // This should actually be voted on..
    pub fn process_update_info(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        link: Option<ContentSource>,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let channel_account_info = next_account_info(accounts_iter)?;
        if channel_account_info.try_data_is_empty()? {
            // Channel does not exist
            return Err(ProgramError::InvalidArgument);
        }
        let mut channel = get_account_data::<ChannelAccount>(program_id, channel_account_info)?;

        let authority_account = next_account_info(accounts_iter)?;
        channel.check_authority(authority_account)?;
        channel.link = link;
        channel.serialize(&mut *channel_account_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<ChannelInstruction>(instruction_data)?;
        match instruction {
            ChannelInstruction::CreateChannel {
                parent,
                name,
                link,
                channel_account_bump_seed,
                channel_authority_config,
                /* ,
                create_rule_address_bump_seed, */
            } => {
                msg!("Instruction: Create channel");
                Self::process_create_channel(
                    program_id,
                    accounts,
                    parent,
                    name,
                    link,
                    channel_authority_config,
                    channel_account_bump_seed, /*
                                               create_rule_address_bump_seed, */
                )
            }
            ChannelInstruction::UpdateInfo { link } => {
                Self::process_update_info(program_id, accounts, link)
            }
            ChannelInstruction::UpdateAuthority(new_authority) => {
                Self::process_update_authority(program_id, accounts, new_authority)
            }
        }
    }
}
