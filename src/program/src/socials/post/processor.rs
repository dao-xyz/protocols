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
    shared::accounts::{check_account_owner, check_system_program},
    socials::{
        channel::state::deserialize_channel_account,
        create_and_serialize_account_signed_verify,
        post::{
            instruction::CreatePostType,
            state::{ActionPost, InformationPost},
        },
        state::AccountType,
        user::state::deserialize_user_account,
    },
    tokens::spl_utils::{
        create_authority_program_address_seeds, create_program_token_account,
        find_authority_program_address, get_token_supply, spl_mint_to, token_transfer,
    },
};

use super::{
    create_escrow_program_address_seeds, create_post_mint_authority_program_address_seeds,
    create_post_mint_program_account, create_rule_associated_program_address_seeds,
    create_treasury_token_account_address_seeds, find_treasury_token_account_address,
    instruction::{CreatePost, PostInstruction, PostVote},
    state::{
        deserialize_action_rule_account, deserialize_post_account, Action, ActionRule,
        ActionStatus, ActionType, PostAccount, PostType, TreasuryAction, VotingRuleUpdate,
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
            Vote::Up,
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
            Vote::Down,
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
                post_type: match post.post_type {
                    CreatePostType::SimplePost => PostType::InformalPost(InformationPost {
                        created_at: timestamp,
                        downvotes: 0,
                        upvotes: 0,
                    }),
                    CreatePostType::ActionPost { action, expires_at } => {
                        if expires_at < timestamp {
                            return Err(ProgramError::InvalidArgument);
                        }
                        PostType::ActionPost(ActionPost {
                            action,
                            created_at: timestamp,
                            downvotes: 0,
                            expires_at: expires_at,
                            status: ActionStatus::Pending,
                            upvotes: 0,
                        })
                    }
                },
                utility_mint_address: post.utility_mint_address,
                channel: post.channel,
                hash: post.hash,
                source: post.source,
                creator: *user_account_info.key,
                asset: super::state::Asset::NonAsset,
                deleted: false,
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
            Vote::Up => mint_upvote_account_info,
            Vote::Down => mint_downvote_account_info,
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
            PostType::InformalPost(mut info) => {
                match stake.vote {
                    Vote::Up => info.upvotes += stake.stake,
                    Vote::Down => info.downvotes += stake.stake,
                };
                PostType::InformalPost(info)
            }
            PostType::ActionPost(mut info) => {
                match stake.vote {
                    Vote::Up => info.upvotes += stake.stake,
                    Vote::Down => info.downvotes += stake.stake,
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
            Vote::Up => mint_upvote_account_info,
            Vote::Down => mint_downvote_account_info,
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
            PostType::InformalPost(mut info) => {
                match stake.vote {
                    Vote::Up => info.upvotes -= stake.stake,
                    Vote::Down => info.downvotes -= stake.stake,
                };
                PostType::InformalPost(info)
            }
            PostType::ActionPost(mut info) => {
                match stake.vote {
                    Vote::Up => info.upvotes -= stake.stake,
                    Vote::Down => info.downvotes -= stake.stake,
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
        let channel = deserialize_channel_account(*channel_account_info.data.borrow())?;
        let governence_mint_info = next_account_info(accounts_iter)?;
        let action_rule_info = next_account_info(accounts_iter)?;
        if action_rule_info.data_is_empty() {
            return Err(ProgramError::InvalidArgument);
        }
        let action_rule = deserialize_action_rule_account(*action_rule_info.data.borrow())?;

        let supply = get_token_supply(governence_mint_info)?;

        check_account_owner(post_account_info, program_id)?;
        check_account_owner(channel_account_info, program_id)?;
        check_account_owner(action_rule_info, program_id)?;

        if &action_rule.channel != channel_account_info.key {
            return Err(ProgramError::InvalidArgument);
        }
        if &post.channel != channel_account_info.key {
            return Err(ProgramError::InvalidArgument);
        }

        if &channel.governence_mint != governence_mint_info.key {
            return Err(ProgramError::InvalidArgument);
        }

        post.post_type = match post.post_type {
            PostType::ActionPost(mut action) => {
                // check if already executed
                if action.status != ActionStatus::Pending {
                    return Err(ProgramError::Custom(123));
                }

                // check if vote is settled
                if action.expires_at <= Clock::get()?.unix_timestamp as u64 {
                    return Err(ProgramError::Custom(124)); // Not ready yet!
                }

                if action_rule.is_approved(&action, supply).unwrap() {
                    action.status = ActionStatus::Approved;
                    match &action.action {
                        Action::DeletePost(post_to_delete) => {
                            let delete_post_account_info = next_account_info(accounts_iter)?;
                            if post_to_delete != delete_post_account_info.key {
                                return Err(ProgramError::InvalidArgument);
                            }
                            let mut post_deleted =
                                deserialize_post_account(*delete_post_account_info.data.borrow())?;
                            post_deleted.deleted = true;
                            post_deleted
                                .serialize(&mut *delete_post_account_info.data.borrow_mut())?;
                        }
                        Action::Treasury(treasury_action) => match treasury_action {
                            TreasuryAction::Transfer {
                                from,
                                to,
                                amount,
                                bump_seed,
                            } => {
                                let from_info = next_account_info(accounts_iter)?;
                                let to_info = next_account_info(accounts_iter)?;
                                if from != from_info.key {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                if to != to_info.key {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                let transfer_authority = next_account_info(accounts_iter)?;
                                let token_program_info = next_account_info(accounts_iter)?;
                                let bump_seeds = &[*bump_seed];
                                let seeds = create_authority_program_address_seeds(
                                    from_info.key,
                                    bump_seeds,
                                );
                                invoke_signed(
                                    &spl_token::instruction::transfer(
                                        token_program_info.key,
                                        from_info.key,
                                        to_info.key,
                                        transfer_authority.key,
                                        &[],
                                        *amount,
                                    )?,
                                    &[
                                        from_info.clone(),
                                        to_info.clone(),
                                        transfer_authority.clone(),
                                        token_program_info.clone(),
                                    ],
                                    &[&seeds],
                                )?;
                            }
                            TreasuryAction::Create { mint } => {
                                let payer_info = next_account_info(accounts_iter)?;
                                let mint_info = next_account_info(accounts_iter)?;
                                if mint == mint_info.key {
                                    let token_account_info = next_account_info(accounts_iter)?;
                                    let token_account_authority_info =
                                        next_account_info(accounts_iter)?;

                                    let system_program_info = next_account_info(accounts_iter)?;
                                    let token_program_info = next_account_info(accounts_iter)?;
                                    let rent_info = next_account_info(accounts_iter)?;

                                    let (treasury_token_address, treasury_token_address_bump_seed) =
                                        find_treasury_token_account_address(
                                            &post.channel,
                                            mint_info.key,
                                            token_program_info.key,
                                            program_id,
                                        );
                                    if &treasury_token_address != token_account_info.key {
                                        return Err(ProgramError::InvalidArgument);
                                    }

                                    let bump_seeds = &[treasury_token_address_bump_seed];
                                    let token_account_seeds =
                                        create_treasury_token_account_address_seeds(
                                            &post.channel,
                                            mint_info.key,
                                            token_program_info.key,
                                            bump_seeds,
                                        );

                                    let (token_account_authority, _) =
                                        find_authority_program_address(
                                            program_id,
                                            &treasury_token_address,
                                        );

                                    if &token_account_authority != token_account_authority_info.key
                                    {
                                        return Err(ProgramError::InvalidArgument);
                                    }

                                    create_program_token_account(
                                        token_account_info,
                                        &token_account_seeds,
                                        mint_info,
                                        token_account_authority_info,
                                        payer_info,
                                        rent_info,
                                        token_program_info,
                                        system_program_info,
                                        program_id,
                                    )?;
                                } else {
                                    return Err(ProgramError::InvalidArgument);
                                }
                            }
                        },
                        Action::ManageRule(modification) => match modification {
                            VotingRuleUpdate::Create { rule, bump_seed } => {
                                let payer_info = next_account_info(accounts_iter)?;
                                let new_rule_info = next_account_info(accounts_iter)?;
                                let system_account = next_account_info(accounts_iter)?;
                                check_system_program(system_account.key)?;
                                let create_rule_bump_seeds = &[*bump_seed];
                                let seeds = create_rule_associated_program_address_seeds(
                                    channel_account_info.key,
                                    &rule.action,
                                    create_rule_bump_seeds,
                                );

                                create_and_serialize_account_signed_verify(
                                    payer_info,
                                    new_rule_info,
                                    &ActionRule {
                                        social_account_type: AccountType::RuleAccount,
                                        account_type: crate::instruction::S2GAccountType::Social,
                                        action: rule.action.clone(),
                                        channel: rule.channel,
                                        info: rule.info.clone(),
                                        name: rule.name.clone(),
                                        criteria: rule.criteria.clone(),
                                        deleted: false,
                                    },
                                    &seeds,
                                    program_id,
                                    system_account,
                                    &Rent::get()?,
                                )?;
                            }
                            VotingRuleUpdate::Delete(rule) => {
                                let rule_info = next_account_info(accounts_iter)?;
                                check_account_owner(rule_info, program_id)?;

                                if rule_info.key != rule {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                let mut rule =
                                    deserialize_action_rule_account(*rule_info.data.borrow())?;
                                if &rule.channel != channel_account_info.key {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                rule.deleted = true;
                                rule.serialize(&mut *rule_info.data.borrow_mut())?;
                            }
                        },
                        Action::CustomEvent { data, event_type } => {
                            // well we dont need to do anything since the data is already on chain and the approved status has/will be set, so integration can be made
                            // but we have to check that the action event_type matches the rule event type
                            // since rules for custom events are controlled by their event type

                            if let ActionType::CustomEvent(expected_event_type) = action_rule.action
                            {
                                if &expected_event_type != event_type {
                                    return Err(ProgramError::InvalidArgument);
                                }
                            } else {
                                // This should not happen, since the rul eaction type will also be of type
                                return Err(ProgramError::InvalidArgument);
                            }
                        }
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
                msg!("Create post execution");
                Self::process_execute_post(program_id, accounts)
            }
        }
    }
}
