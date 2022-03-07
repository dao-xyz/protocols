use crate::{
    accounts::AccountType,
    error::PostError,
    instruction::{CreatePostType, PostVote},
    state::post::{
        deserialize_post_account, InformationPost, PostAccount, PostType, VotingRuleUpdate,
    },
    state::{
        enums::TransactionExecutionStatus,
        proposal::{
            proposal_option::get_proposal_option_data,
            proposal_transaction::get_proposal_transaction_data_for_proposal,
        },
        rules::rule::{get_rule_data, Rule},
        token_owner_record::{
            get_token_owner_record_address_seeds, get_token_owner_record_data_for_owner,
            TokenOwnerRecordV2,
        },
        vote_record::{get_vote_record_address_seeds, Vote, VoteRecordV2},
    },
};
use borsh::BorshSerialize;
use luser::state::deserialize_user_account;
use shared::account::{create_and_serialize_account_signed, get_account_data};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
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

use super::{
    create_escrow_program_address_seeds, create_post_mint_authority_program_address_seeds,
};
pub fn process_create_post_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    /*  amount: u64, */
    //proposal_vote_weight_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let mut post = deserialize_post_account(&post_account_info.data.borrow())?;

    let vote_record_info = next_account_info(accounts_iter)?;
    /*     let proposal_vote_weight_info = next_account_info(accounts_iter)?; */
    let token_owner_record_info = next_account_info(accounts_iter)?;
    let voter_token_owner_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    /*   let spl_token_info = next_account_info(accounts_iter)?;
    let rent_sysvar_info = next_account_info(accounts_iter)?; */
    let rent = Rent::get()?;

    /*    let payer_account = next_account_info(accounts_iter)?;
    let payer_governence_token_account = next_account_info(accounts_iter)?;

    let system_account = next_account_info(accounts_iter)?;
    let rent_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    let spl_associated_token_acount_program_info = next_account_info(accounts_iter)?; */

    // TODO: CHECK OWNER OF POST, CHECK MINTS,
    post.post_type = match post.post_type {
        PostType::InformationPost(mut info) => {
            /*  match vote.vote {
                Vote::Up => info.upvotes += stake.stake,
                Vote::Down => info.downvotes += stake.stake,
            }; */
            PostType::InformationPost(info)
        }
        PostType::Proposal(proposal) => {
            if !vote_record_info.data_is_empty() {
                return Err(PostError::VoteAlreadyExists.into());
            }
            let rule_info = next_account_info(accounts_iter)?;
            let rule = get_rule_data(program_id, rule_info, &post.channel)?;

            let token_owner_record_data = get_token_owner_record_data_for_owner(
                program_id,
                token_owner_record_info,
                voter_token_owner_record_info,
            )?;

            // While we have rule we want to vote on, continue
            /*  let rule_vote_weight_info = next_account_info(accounts_iter); */
            let vote = Vec::new();
            let mut option_info_next = next_account_info(accounts_iter);
            while let Ok(option_info) = option_info_next {
                // Vote with rule weight
                /*              let rule_weight_info = next_account_info(accounts_iter)?;
                let rule_weight =
                    get_rule_vote_weight_data(program_id, rule_weight_info, rule_info_ok.key)?; */

                // Check create vote record
                if !vote_record_info.data_is_empty() {
                    return Err(PostError::VoteAlreadyExists.into());
                }

                let mut option_data =
                    get_proposal_option_data(program_id, option_info, post_account_info.key)?;

                option_data.add_weight(
                    token_owner_record_data.governing_token_deposit_amount,
                    &token_owner_record_data.governing_token_mint,
                    rule_info.key,
                    &rule,
                )?;
                option_data.serialize(&mut *option_info.data.borrow_mut())?;
                option_info_next = next_account_info(accounts_iter);
                // Vote
                /* let proposal_vote_weight_info = next_account_info(accounts_iter)?;
                if proposal_vote_weight_info.data_is_empty() {
                    // Create weight
                    let proposal_vote_weight_data = ProposalVoteWeight {
                        account_type: AccountType::ProposalVoteWeight,
                        proposal: *post_account_info.key,
                        rule: *rule_info_ok.key,
                        weight: 0,
                        option_index: *option_index,
                    };

                    proposal_vote_weight_data.add_weight(
                        voter_weight,
                        &token_owner_record_data.governing_token_mint,
                        &rule_weight,
                    );

                    create_and_serialize_account_signed::<ProposalVoteWeight>(
                        payer_info,
                        proposal_vote_weight_info,
                        &proposal_vote_weight_data,
                        &create_proposal_vote_weight_program_address_seeds(
                            post_account_info.key,
                            voter_token_owner_record_info.key,
                            &option_index.to_le_bytes(),
                            &[proposal_vote_weight_bump_seed],
                        ),
                        program_id,
                        system_info,
                        &rent,
                    )?;
                } else {
                    // Update weight
                    let mut proposal_vote_weight_data = get_proposal_vote_weight_data(
                        program_id,
                        proposal_vote_weight_info,
                        post_account_info.key,
                        rule_info_ok.key,
                        option_index,
                    )?;
                    proposal_vote_weight_data.add_weight(
                        voter_weight,
                        &token_owner_record_data.governing_token_mint,
                        &rule_weight,
                    );
                    proposal_vote_weight_data
                        .serialize(&mut *proposal_vote_weight_info.data.borrow_mut());
                } */
            }

            proposal.assert_valid_vote(&vote)?;

            // Add vote record so we can not vote again through the same rule
            let vote_record_data = VoteRecordV2 {
                account_type: AccountType::VoteRecordV2,
                post: *post_account_info.key,
                governing_token_owner: *voter_token_owner_record_info.key,
                vote,
                rule: *rule_info.key,
                is_relinquished: false,
            };

            create_and_serialize_account_signed::<VoteRecordV2>(
                payer_info,
                vote_record_info,
                &vote_record_data,
                &get_vote_record_address_seeds(
                    post_account_info.key,
                    voter_token_owner_record_info.key,
                    rule_info.key,
                ),
                program_id,
                system_info,
                &rent,
            )?;

            // Update propsal

            PostType::Proposal(proposal)
        }
    };

    post.serialize(&mut *post_account_info.data.borrow_mut())?;

    Ok(())
}

/* pub fn process_proposal_vote(proposal: &ProposalV2) {
    proposal.assert_can_cast_vote(config, current_unix_timestamp)
} */
/*
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

    // TODO: CHECK OWNER OF POST, CHECK MINTS,

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
    let seeds = create_post_mint_authority_program_address_seeds(post_account_info.key, bump_seeds);

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
        PostType::InformationPost(mut info) => {
            match stake.vote {
                Vote::Up => info.upvotes -= stake.stake,
                Vote::Down => info.downvotes -= stake.stake,
            };
            PostType::InformationPost(info)
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
 */
