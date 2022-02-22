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

use luser::{create_and_serialize_account_signed_verify, state::deserialize_user_account};

use crate::shared::names::entity_name_is_valid;

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
        creator: Pubkey, // a
        governence_mint: Pubkey,
        name: String,
        link: Option<ContentSource>,
        channel_account_bump_seed: u8, /*
                                       create_rule_address_bump_seed: u8, */
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let creator_user_account_info = next_account_info(accounts_iter)?;
        let creator_user =
            deserialize_user_account(creator_user_account_info.data.borrow().as_ref())?;
        if &creator_user.owner != payer_account.key {
            return Err(ProgramError::IllegalOwner); // To prevent someone to create a channel for someone else
        }

        if &creator != creator_user_account_info.key {
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
                creator,
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

        // Create initial rules (the only rule we need to create is that it is possible to create more rules)            payer_account,
        /* let create_rule_bump_seeds = &[create_rule_address_bump_seed];
        let rule_type = ActionType::ManageRule(RuleUpdateType::Create);
        let create_rule_seeds = create_rule_associated_program_address_seeds(
            channel_account_info.key,
            &rule_type,
            create_rule_bump_seeds,
        ); */

        // Create a rule with acceptance criteria on the channel that allows
        // proposals to made to create other rules
        /*  create_and_serialize_account_signed_verify(
            payer_account,
            new_rule_account_info,
            &ActionRule {
                social_account_type: AccountType::RuleAccount,
                account_type: crate::instruction::lchannelAccountType::Social,
                action: ActionType::ManageRule(RuleUpdateType::Create).clone(),
                channel: *channel_account_info.key,
                info: None, // Does not matter, rule is self evident
                name: None, // Does not matter, rule is self evident
                criteria: AcceptenceCriteria::default(),
                deleted: false,
            },
            &create_rule_seeds,
            program_id,
            system_account,
            &Rent::get()?,
        )?; */

        Ok(())
    }

    // This should actually be voted on..
    pub fn process_update_channel(
        accounts: &[AccountInfo],
        link: Option<ContentSource>,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let creator_user_account_info = next_account_info(accounts_iter)?;
        let creator_user =
            deserialize_user_account(creator_user_account_info.data.borrow().as_ref())?;
        if &creator_user.owner != payer_account.key {
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
        if &channel.creator != creator_user_account_info.key {
            msg!(
                "Expected creator {} but got {}",
                channel.creator,
                creator_user_account_info.key
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
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<ChannelInstruction>(instruction_data)?;
        match instruction {
            ChannelInstruction::CreateChannel {
                creator,
                governence_mint,
                name,
                link,
                channel_account_bump_seed, /* ,
                                           create_rule_address_bump_seed, */
            } => {
                msg!("Instruction: Create channel");
                Self::process_create_channel(
                    program_id,
                    accounts,
                    creator,
                    governence_mint,
                    name,
                    link,
                    channel_account_bump_seed, /*
                                               create_rule_address_bump_seed, */
                )
            }
            ChannelInstruction::UpdateChannel { link } => {
                Self::process_update_channel(accounts, link)
            }
        }
    }
}
