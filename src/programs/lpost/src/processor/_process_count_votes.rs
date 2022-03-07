use crate::{
    accounts::AccountType,
    error::PostError,
    instruction::{CreatePostType, PostVote},
    state::post::{
        deserialize_post_account, InformationPost, PostAccount, PostType, VotingRuleUpdate,
    },
    state::{
        enums::TransactionExecutionStatus,
        proposal::ProposalV2,
        proposal_transaction::{
            get_proposal_transaction_address_seeds, get_proposal_transaction_data_for_proposal,
            InstructionData, ProposalTransactionV2,
        },
        rules::{rule::Rule, rule_vote_weight::RuleVoteWeight},
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
    vote: Vote,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let proposal_transaction_info = next_account_info(accounts_iter)?;
    let vote_record_info = next_account_info(accounts_iter)?;
    let token_owner_record_info = next_account_info(accounts_iter)?;
    let voter_token_owner_record_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;
    /*   let spl_token_info = next_account_info(accounts_iter)?;
    let rent_sysvar_info = next_account_info(accounts_iter)?; */
    let rent = Rent::get()?;

    let mut post = deserialize_post_account(&post_account_info.data.borrow())?;
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
            let token_owner_record_data = get_token_owner_record_data_for_owner(
                program_id,
                token_owner_record_info,
                voter_token_owner_record_info,
            )?;

            // try to tip vote
            // XXX

            // Create and serialize VoteRecord
            let vote_record_data = VoteRecordV2 {
                account_type: AccountType::VoteRecordV2,
                post: *post_account_info.key,
                governing_token_owner: *voter_token_owner_record_info.key,
                vote,
                is_relinquished: false,
            };

            create_and_serialize_account_signed::<VoteRecordV2>(
                payer_info,
                vote_record_info,
                &vote_record_data,
                &get_vote_record_address_seeds(
                    post_account_info.key,
                    voter_token_owner_record_info.key,
                ),
                program_id,
                system_info,
                &rent,
            )?;

            // Update propsal
            let mut proposal_transaction_data = get_proposal_transaction_data_for_proposal(
                program_id,
                proposal_transaction_info,
                post_account_info.key,
            )?;
            let mut rule_vote_weight_info = next_account_info(accounts_iter)?;
            let mut rule_vote_weight =
                get_account_data::<RuleVoteWeight>(program_id, rule_vote_weight_info)?;
            for instruction in &mut proposal_transaction_data.instructions {
                if let Some(delegation_rule) = &token_owner_record_data.delegated_by_rule {
                    if delegation_rule != &instruction.rule {
                        continue; // This token owner record can only be used for voting based on the delegated rul
                    }
                }

                while rule_vote_weight_info.key != &instruction.rule {
                    rule_vote_weight_info = next_account_info(accounts_iter)?;
                    rule_vote_weight =
                        get_account_data::<RuleVoteWeight>(program_id, rule_vote_weight_info)?;
                }

                instruction.add_weight(
                    token_owner_record_data.governing_token_deposit_amount,
                    &token_owner_record_data.governing_token_mint,
                    &rule_vote_weight,
                )?;
            }

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
