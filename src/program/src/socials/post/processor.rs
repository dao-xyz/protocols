use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_associated_token_account::create_associated_token_account;
use spl_token::instruction::burn;

use crate::{
    shared::accounts::check_account_owner,
    socials::{
        channel::state::deserialize_channel_account, create_and_serialize_account_signed_verify,
        post::state::InformationPost, state::AccountType, user::state::deserialize_user_account,
    },
    tokens::spl_utils::{
        create_program_token_account, get_token_supply, spl_mint_to, token_transfer,
    },
};

use super::{
    create_escrow_program_address_seeds, create_post_mint_authority_program_address_seeds,
    create_post_mint_program_account,
    instruction::{CreatePost, PostInstruction, PostVote},
    state::{
        deserialize_action_rule_account, deserialize_post_account, Action, ActionStatus,
        PostAccount, PostType,
    },
    Vote,
};

pub struct Processor {}
impl Processor {
    // Create post

    pub fn process_create_post(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        post: CreatePost,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let user_account_info = next_account_info(accounts_iter)?;
        let user = deserialize_user_account(user_account_info.data.borrow().as_ref())?;
        if &user.owner != payer_account.key {
            // Can not create a post for another user
            return Err(ProgramError::InvalidArgument);
        }

        let post_account_info = next_account_info(accounts_iter)?;
        let mint_upvote_account_info = next_account_info(accounts_iter)?;
        let mint_downvote_account_info = next_account_info(accounts_iter)?;
        let mint_authority_account_info = next_account_info(accounts_iter)?;
        let governence_mint_account_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;
        let rent_info = next_account_info(accounts_iter)?;
        let token_program_info = next_account_info(accounts_iter)?;
        let rent = Rent::get()?;
        let content_hash = post.hash;

        // Upvote tokens
        create_post_mint_program_account(
            post_account_info.key,
            Vote::UP,
            mint_upvote_account_info,
            post.mint_upvote_bump_seed,
            mint_authority_account_info,
            payer_account,
            rent_info,
            token_program_info,
            system_account,
            program_id,
        )?;

        // Downvote tokens
        create_post_mint_program_account(
            post_account_info.key,
            Vote::DOWN,
            mint_downvote_account_info,
            post.mint_downvote_bump_seed,
            mint_authority_account_info,
            payer_account,
            rent_info,
            token_program_info,
            system_account,
            program_id,
        )?;

        let escrow_utility_token_account_info = next_account_info(accounts_iter)?;

        // create empty escrow account
        let escrow_bump_seeds = &[post.escrow_bump_seed];
        let escrow_account_seeds =
            create_escrow_program_address_seeds(post_account_info.key, escrow_bump_seeds);
        let expected_escrow_address =
            Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

        if escrow_utility_token_account_info.key != &expected_escrow_address {
            msg!(
                "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                escrow_utility_token_account_info.key,
                expected_escrow_address
            );
            return Err(ProgramError::InvalidSeeds);
        }
        msg!("Create escrow account");
        create_program_token_account(
            escrow_utility_token_account_info,
            &escrow_account_seeds,
            governence_mint_account_info,
            mint_authority_account_info,
            payer_account,
            rent_info,
            token_program_info,
            system_account,
            program_id,
        )?;
        let timestamp = Clock::get()?.unix_timestamp as u64;

        create_and_serialize_account_signed_verify(
            payer_account,
            post_account_info,
            &PostAccount {
                account_type: crate::instruction::S2GAccountType::Social,
                social_account_type: AccountType::PostAccount,
                post_type: PostType::SimplePost(InformationPost {
                    created_at: timestamp,
                    updated_at: timestamp,
                    downvotes: 0,
                    upvotes: 0,
                }),
                utility_mint_address: post.utility_mint_address,
                channel: post.channel,
                hash: post.hash,
                source: post.source,
                creator: *user_account_info.key,
                asset: super::state::Asset::NonAsset,
            },
            &[&content_hash, &[post.post_bump_seed]],
            program_id,
            system_account,
            &rent,
        )?;

        Ok(())
    }

    pub fn process_create_post_vote(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        stake: PostVote,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let payer_governence_token_account = next_account_info(accounts_iter)?;
        let post_account_info = next_account_info(accounts_iter)?;
        let mut post = deserialize_post_account(&post_account_info.data.borrow())?;
        let mint_upvote_account_info = next_account_info(accounts_iter)?;
        let mint_downvote_account_info = next_account_info(accounts_iter)?;
        let mint_authority_account_info = next_account_info(accounts_iter)?;
        let mint_associated_token_account = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;
        let rent_info = next_account_info(accounts_iter)?;
        let token_program_info = next_account_info(accounts_iter)?;
        let spl_associated_token_acount_program_info = next_account_info(accounts_iter)?;

        let mint_account_info = match stake.vote {
            Vote::UP => mint_upvote_account_info,
            Vote::DOWN => mint_downvote_account_info,
        };

        if mint_associated_token_account.data.borrow().is_empty() {
            // Unitialized token account
            // this will cost some sol, but we assume we don't have to mint tokens for this
            msg!(
                "Create vote token account {}",
                mint_associated_token_account.key
            );

            invoke(
                &create_associated_token_account(
                    payer_account.key,
                    payer_account.key,
                    mint_account_info.key,
                ),
                &[
                    payer_account.clone(),
                    mint_associated_token_account.clone(),
                    payer_account.clone(),
                    mint_account_info.clone(),
                    system_account.clone(),
                    token_program_info.clone(),
                    rent_info.clone(),
                    spl_associated_token_acount_program_info.clone(),
                ],
            )?;
        }

        let escrow_token_account_info = next_account_info(accounts_iter)?;
        let escrow_bump_seeds = &[stake.escrow_bump_seed];
        let escrow_account_seeds =
            create_escrow_program_address_seeds(post_account_info.key, escrow_bump_seeds);
        let expected_escrow_address =
            Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

        if escrow_token_account_info.key != &expected_escrow_address {
            msg!(
                "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                escrow_token_account_info.key,
                expected_escrow_address
            );
            return Err(ProgramError::InvalidSeeds);
        }

        token_transfer(
            token_program_info.clone(),
            payer_governence_token_account.clone(),
            escrow_token_account_info.clone(),
            payer_account.clone(),
            stake.stake,
        )?;

        // for some tokens (Upvotes or downvotes depending on the mint info)
        spl_mint_to(
            mint_associated_token_account,
            mint_account_info,
            mint_authority_account_info,
            &create_post_mint_authority_program_address_seeds(
                post_account_info.key,
                &[stake.mint_authority_bump_seed],
            ),
            stake.stake,
            program_id,
        )?;

        post.post_type = match post.post_type {
            PostType::SimplePost(mut info) => {
                match stake.vote {
                    Vote::UP => info.upvotes += stake.stake,
                    Vote::DOWN => info.downvotes += stake.stake,
                };
                PostType::SimplePost(info)
            }
            PostType::ActionPost(mut info) => {
                match stake.vote {
                    Vote::UP => info.upvotes += stake.stake,
                    Vote::DOWN => info.downvotes += stake.stake,
                };
                PostType::ActionPost(info)
            }
        };

        post.serialize(&mut *post_account_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_create_post_unvote(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        stake: PostVote,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let payer_governence_token_account = next_account_info(accounts_iter)?;
        let post_account_info = next_account_info(accounts_iter)?;
        let mut post = deserialize_post_account(&post_account_info.data.borrow())?;
        let mint_upvote_account_info = next_account_info(accounts_iter)?;
        let mint_downvote_account_info = next_account_info(accounts_iter)?;
        let mint_authority_account_info = next_account_info(accounts_iter)?;
        let mint_associated_token_account = next_account_info(accounts_iter)?;
        let token_program_info = next_account_info(accounts_iter)?;

        let mint_account_info = match stake.vote {
            Vote::UP => mint_upvote_account_info,
            Vote::DOWN => mint_downvote_account_info,
        };

        let escrow_token_account_info = next_account_info(accounts_iter)?;
        let escrow_bump_seeds = &[stake.escrow_bump_seed];
        let escrow_account_seeds =
            create_escrow_program_address_seeds(post_account_info.key, escrow_bump_seeds);

        let expected_escrow_address =
            Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

        if escrow_token_account_info.key != &expected_escrow_address {
            msg!(
                "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                escrow_token_account_info.key,
                expected_escrow_address
            );
            return Err(ProgramError::InvalidSeeds);
        }

        let bump_seeds = &[stake.mint_authority_bump_seed];
        let seeds =
            create_post_mint_authority_program_address_seeds(post_account_info.key, bump_seeds);

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_info.key,
                escrow_token_account_info.key,
                payer_governence_token_account.key,
                mint_authority_account_info.key,
                &[],
                stake.stake,
            )?,
            &[
                escrow_token_account_info.clone(),
                payer_governence_token_account.clone(),
                mint_authority_account_info.clone(),
                token_program_info.clone(),
            ],
            &[&seeds],
        )?;

        invoke(
            &burn(
                token_program_info.key,
                mint_associated_token_account.key,
                mint_account_info.key,
                payer_account.key,
                &[],
                stake.stake,
            )?,
            &[
                mint_associated_token_account.clone(),
                mint_account_info.clone(),
                payer_account.clone(),
                token_program_info.clone(),
            ],
        )?;

        post.post_type = match post.post_type {
            PostType::SimplePost(mut info) => {
                match stake.vote {
                    Vote::UP => info.upvotes -= stake.stake,
                    Vote::DOWN => info.downvotes -= stake.stake,
                };
                PostType::SimplePost(info)
            }
            PostType::ActionPost(mut info) => {
                match stake.vote {
                    Vote::UP => info.upvotes -= stake.stake,
                    Vote::DOWN => info.downvotes -= stake.stake,
                };
                PostType::ActionPost(info)
            }
        };

        post.serialize(&mut *post_account_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_execute_post(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let post_account_info = next_account_info(accounts_iter)?;
        let mut post = deserialize_post_account(*post_account_info.data.borrow())?;
        let channel_account_info = next_account_info(accounts_iter)?;
        let action_rule_info = next_account_info(accounts_iter)?;
        let action_rule = deserialize_action_rule_account(*action_rule_info.data.borrow())?;
        let utility_mint_info = next_account_info(accounts_iter)?;
        let supply = get_token_supply(utility_mint_info)?;
        let channel = deserialize_channel_account(*channel_account_info.data.borrow())?;

        check_account_owner(post_account_info, program_id)?;
        check_account_owner(channel_account_info, program_id)?;
        check_account_owner(action_rule_info, program_id)?;

        if &action_rule.channel != channel_account_info.key {
            return Err(ProgramError::InvalidArgument);
        }
        if &post.channel != channel_account_info.key {
            return Err(ProgramError::InvalidArgument);
        }

        post.post_type = match post.post_type {
            PostType::ActionPost(mut action) => {
                // check if vote is settled
                if action_rule.is_approved(&action, supply).unwrap() {
                    action.status = ActionStatus::Approved;
                    match action.action {
                        Action::DeletePost(post) => {}
                        Action::Event(x) => {}
                        Action::ManageRule() => {}
                        Action::SelfDestruct() => {}
                        Action::TransferTreasury() => {}
                    }
                } else {
                    action.status = ActionStatus::Rejected;
                }
                PostType::ActionPost(action)
            }
            _ => {
                panic!("Can not execute a non action post")
            }
        };

        post.serialize(&mut *post_account_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: PostInstruction,
    ) -> ProgramResult {
        match instruction {
            PostInstruction::CreatePost(post) => {
                msg!("Create post");
                Self::process_create_post(program_id, accounts, post)
            }
            /* ChatInstruction::CreatePostContent(content) => {
                msg!("Create post content");
                Self::process_create_post_content(program_id, accounts, content)
            } */
            PostInstruction::Vote(stake) => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Create vote");
                Self::process_create_post_vote(program_id, accounts, stake)
            }

            PostInstruction::Unvote(stake) => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Create unvote");
                Self::process_create_post_unvote(program_id, accounts, stake)
            }

            PostInstruction::ExecutePost => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Create unvote");
                Self::process_execute_post(program_id, accounts)
            }
        }
    }
}
