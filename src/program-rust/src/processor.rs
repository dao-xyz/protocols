use borsh::BorshDeserialize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    account::create_and_serialize_account_signed_verify_with_bump,
    accounts::{deserialize_post_account, PostAccount, PostContentAccount},
};
use crate::{
    account::{create_and_serialize_account_signed, create_and_serialize_account_signed_verify},
    accounts::{deserialize_user_account, AccountContainer, MessageAccount},
    address::generate_seeds_from_string,
    instruction::ChatInstruction,
    rates::get_allowed_mint_amount,
    spl_utils::{
        create_escrow_account_bump_seeds, create_program_mint_account,
        create_user_post_token_account, spl_mint_to, transfer_to,
    },
};
use solana_program::system_instruction::{allocate, create_account};

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;

// Program entrypoint's implementation
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = ChatInstruction::try_from_slice(input)?;

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let system_account = next_account_info(accounts_iter)?;
    let _program_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;

    match instruction {
        ChatInstruction::CreateUser(user) => {
            if user.name.is_empty() {
                return Err(ProgramError::InvalidArgument);
            }
            // check if leading or trailing spaces, if so name is invalid
            let mut chars = user.name.chars();
            if chars.next().unwrap().is_whitespace() || chars.last().unwrap_or('a').is_whitespace()
            {
                return Err(ProgramError::InvalidArgument);
            }

            if &user.owner != payer_account.key {
                return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
            }

            let user_acount_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;
            let seeds = generate_seeds_from_string(&user.name)?;
            let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
            create_and_serialize_account_signed(
                payer_account,
                user_acount_info,
                &AccountContainer::UserAccount(user),
                seed_slice,
                program_id,
                system_account,
                &rent,
            )?;
        }

        ChatInstruction::CreateChannel(channel) => {
            let user_account_info = next_account_info(accounts_iter)?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key {
                return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
            }

            let channel_account_info = next_account_info(accounts_iter)?;

            let rent = Rent::get()?;
            let seeds = generate_seeds_from_string(&channel.name)?;
            let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
            create_and_serialize_account_signed(
                payer_account,
                channel_account_info,
                &AccountContainer::ChannelAccount(channel),
                seed_slice,
                program_id,
                system_account,
                &rent,
            )?;
        }
        ChatInstruction::UpdateChannel(_) => {
            /*  let channel_account_info = next_account_info(accounts_iter)?;

            // Don't allow channel name to be updated, since it would require us to resize the account size
            // This would also mean that the PDA would change!
            channel.serialize(&mut *channel_account_info.data.borrow_mut())? */
        }

        ChatInstruction::SendMessage(send_message) => {
            // Initializes an account for us that lets us build an message
            let user_account_info = next_account_info(accounts_iter)?;
            let channel_account_info = next_account_info(accounts_iter)?;
            let message_account = MessageAccount::new(
                send_message.user,
                send_message.channel,
                send_message.timestamp,
                send_message.message,
            );
            let message_account_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key {
                return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
            }
            create_and_serialize_account_signed_verify_with_bump(
                payer_account,
                message_account_info,
                &AccountContainer::MessageAccount(message_account),
                &[
                    &user_account_info.key.to_bytes(),
                    &channel_account_info.key.to_bytes(),
                    &send_message.timestamp.to_le_bytes(),
                ],
                program_id,
                system_account,
                &rent,
                send_message.bump_seed,
            )?;
        }

        ChatInstruction::CreatePost(post) => {
            //let token_account_info = next_account_info(accounts_iter)?;
            let user_account_info = next_account_info(accounts_iter)?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key {
                // Can not create a post for another user
                return Err(ProgramError::InvalidArgument);
            }

            let post_account_info = next_account_info(accounts_iter)?;
            let escrow_account_info = next_account_info(accounts_iter)?;
            let mint_account_info = next_account_info(accounts_iter)?;
            let mint_authority_account_info = next_account_info(accounts_iter)?;
            let user_post_token_account_info = next_account_info(accounts_iter)?;

            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;

            create_and_serialize_account_signed_verify_with_bump(
                payer_account,
                post_account_info,
                &AccountContainer::PostAccount(PostAccount {
                    channel: post.channel,
                    content: post.content,
                    spread_factor: post.spread_factor,
                    timestamp: post.timestamp,
                    token: *mint_account_info.key,
                    user: *user_account_info.key,
                }),
                &[
                    user_account_info.key.as_ref(),
                    post.channel.as_ref(),
                    &post.timestamp.to_le_bytes(),
                ],
                program_id,
                system_account,
                &rent,
                post.post_bump_seed,
            )?;

            create_program_mint_account(
                post_account_info.key,
                mint_account_info,
                post.mint_bump_seed,
                mint_authority_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // create empty escrow account
            let escrow_bump_seeds = &[post.escrow_account_bump_seed];
            let escrow_account_seeds =
                create_escrow_account_bump_seeds(&post_account_info.key, escrow_bump_seeds);
            let expected_escrow_address =
                Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

            if escrow_account_info.key != &expected_escrow_address {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    escrow_account_info.key,
                    expected_escrow_address
                );
                return Err(ProgramError::InvalidSeeds);
            }
            let minimum_balance_as_stake = rent.minimum_balance(0);

            let create_account_instruction = create_account(
                payer_account.key,
                escrow_account_info.key,
                minimum_balance_as_stake,
                0 as u64,
                program_id,
            );
            invoke_signed(
                &create_account_instruction,
                &[
                    payer_account.clone(),
                    escrow_account_info.clone(),
                    system_account.clone(),
                ],
                &[&escrow_account_seeds],
            )?;

            // create user stake account
            create_user_post_token_account(
                &user_account_info.key,
                &post_account_info.key,
                user_post_token_account_info,
                post.user_post_token_account_bump_seed,
                mint_account_info,
                mint_authority_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // Mint for the minimum balance, as we hade to put some balance into the escrow account
            spl_mint_to(
                user_post_token_account_info,
                mint_account_info,
                mint_authority_account_info,
                post.mint_authority_bump_seed,
                get_allowed_mint_amount(
                    escrow_account_info,
                    minimum_balance_as_stake,
                    post.spread_factor,
                ),
                program_id,
            )?;
        }
        ChatInstruction::CreatePostContent(content) => {
            let post_content_account_info = next_account_info(accounts_iter)?;
            let hash = content.message.hash();
            let rent = Rent::get()?;
            // If creation can be signed hash and bump seed, we know hash is correct
            create_and_serialize_account_signed_verify_with_bump(
                payer_account,
                post_content_account_info,
                &AccountContainer::PostContentAccount(PostContentAccount {
                    message: content.message,
                }),
                &[&hash],
                program_id,
                system_account,
                &rent,
                content.bump_seed,
            )?;
        }

        ChatInstruction::StakePost(stake) => {
            //let token_account_info = next_account_info(accounts_iter)?;
            msg!("New stake");
            msg!(stake.stake.to_string().as_str());
            let post_account_info = next_account_info(accounts_iter)?;
            let post_account = deserialize_post_account(post_account_info.data.borrow().as_ref());
            let escrow_account_info = next_account_info(accounts_iter)?;
            let mint_account_info = next_account_info(accounts_iter)?;
            let mint_authority_account_info = next_account_info(accounts_iter)?;
            let user_post_token_account_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;

            // Verify escrow account is correct
            let escrow_bump_seeds = &[stake.escrow_account_bump_seed];
            let escrow_account_seeds =
                create_escrow_account_bump_seeds(&stake.post, escrow_bump_seeds);
            let expected_escrow_address =
                Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

            if escrow_account_info.key != &expected_escrow_address {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    escrow_account_info.key,
                    expected_escrow_address
                );
                return Err(ProgramError::InvalidSeeds);
            }

            create_user_post_token_account(
                &stake.user,
                &stake.post,
                user_post_token_account_info,
                stake.user_post_token_account_bump_seed,
                mint_account_info,
                mint_authority_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // deduct SOL
            transfer_to(payer_account, escrow_account_info, stake.stake)?;

            // for some LIKES
            spl_mint_to(
                user_post_token_account_info,
                mint_account_info,
                mint_authority_account_info,
                stake.mint_authority_bump_seed,
                get_allowed_mint_amount(
                    escrow_account_info,
                    stake.stake,
                    post_account.spread_factor,
                ),
                program_id,
            )?;
        }
    }

    Ok(())
}
