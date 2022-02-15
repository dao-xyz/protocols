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
        create_and_serialize_account_signed_verify, state::AccountType,
        user::state::deserialize_user_account,
    },
};

use super::{
    create_channel_account_program_address_seeds,
    instruction::ChannelInstruction,
    state::{deserialize_channel_account, ChannelAccount},
};

pub struct Processor {}
impl Processor {
    // Program entrypoint's implementation

    pub fn process_create_channel(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owner: Pubkey, // a
        governence_mint: Pubkey,
        name: String,
        link: Option<String>,
        channel_account_bump_seed: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let owner_user_account_info = next_account_info(accounts_iter)?;
        let owner_user = deserialize_user_account(owner_user_account_info.data.borrow().as_ref())?;
        if &owner_user.owner != payer_account.key {
            return Err(ProgramError::IllegalOwner); // To prevent someone to create a channel for someone else
        }
        if &owner != owner_user_account_info.key {
            return Err(ProgramError::IllegalOwner);
        }
        if !entity_name_is_valid(name.as_ref()) {
            return Err(ProgramError::InvalidArgument);
        }

        let channel_account_info = next_account_info(accounts_iter)?;
        if !channel_account_info.try_data_is_empty()? {
            // Channel already exist
            return Err(ProgramError::InvalidAccountData);
        }

        let system_account = next_account_info(accounts_iter)?;
        let rent = Rent::get()?;
        /*
           Channel and user names must be unique, as we generate the seeds in the same way for both
           Do we want this really?
        */
        let mut seeds = create_channel_account_program_address_seeds(name.as_ref())?;
        seeds.push(vec![channel_account_bump_seed]);
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
        create_and_serialize_account_signed_verify(
            payer_account,
            channel_account_info,
            &ChannelAccount {
                account_type: crate::instruction::S2GAccountType::Social,
                social_account_type: AccountType::ChannelAccount,
                owner,
                governence_mint,
                link,
                name,
                creation_timestamp: Clock::get()?.unix_timestamp as u64,
            },
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_update_channel(accounts: &[AccountInfo], link: Option<String>) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let owner_user_account_info = next_account_info(accounts_iter)?;
        let owner_user = deserialize_user_account(owner_user_account_info.data.borrow().as_ref())?;
        if &owner_user.owner != payer_account.key {
            return Err(ProgramError::IllegalOwner); // To prevent someone to create a channel for someone else
        }
        let channel_account_info = next_account_info(accounts_iter)?;
        if channel_account_info.try_data_is_empty()? {
            // Channel does not exist
            return Err(ProgramError::InvalidArgument);
        }

        if !payer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut channel = deserialize_channel_account(*channel_account_info.data.borrow())?;
        if &channel.owner != owner_user_account_info.key {
            msg!(
                "Expected owner {} but got {}",
                channel.owner,
                owner_user_account_info.key
            );
            return Err(ProgramError::IllegalOwner);
        }
        channel.link = link;
        channel.serialize(&mut *channel_account_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: ChannelInstruction,
    ) -> ProgramResult {
        match instruction {
            ChannelInstruction::CreateChannel {
                owner,
                governence_mint,
                name,
                link,
                channel_account_bump_seed,
            } => {
                msg!("Instruction: Create channel");
                Self::process_create_channel(
                    program_id,
                    accounts,
                    owner,
                    governence_mint,
                    name,
                    link,
                    channel_account_bump_seed,
                )
            }
            ChannelInstruction::UpdateChannel { link } => {
                Self::process_update_channel(accounts, link)
            }
        }
    }
}
