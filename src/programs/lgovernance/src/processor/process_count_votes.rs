use crate::{
    error::PostError,
    state::{
        enums::ProposalState,
        post::{deserialize_post_account, PostType},
        proposal::VoteType,
    },
    state::{
        proposal::{proposal_option::get_proposal_option_data, OptionVoteResult},
        rules::rule::get_rule_data,
    },
    tokens::spl_utils::get_spl_token_mint_supply,
};

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_count_votes(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let mut post = deserialize_post_account(&post_account_info.data.borrow())?;
    let proposal_option_info = next_account_info(accounts_iter)?;
    let proposal_deny_option_info = next_account_info(accounts_iter)?;

    let proposal_deny_option_data =
        get_proposal_option_data(program_id, proposal_deny_option_info, post_account_info.key)?;
    let mut proposal_option_data =
        get_proposal_option_data(program_id, proposal_option_info, post_account_info.key)?;

    if proposal_option_data.vote_result != OptionVoteResult::None {
        return Err(PostError::VotesAlreadyCounted.into());
    }

    if let PostType::Proposal(proposal) = &mut post.post_type {
        if proposal.max_vote_weights_calculated_at.is_none() {
            return Err(PostError::MaxWeightsNotCalculated.into());
        }

        for (i, max_rule_weight) in proposal.rules_max_vote_weight.iter().enumerate() {
            let option_vote_weight = proposal_option_data.vote_weights.get(i).unwrap();
            let deny_option_vote_weight = proposal_deny_option_data.vote_weights.get(i).unwrap();
            if max_rule_weight.rule != option_vote_weight.rule
                || max_rule_weight.rule != deny_option_vote_weight.rule
            {
                return Err(PostError::InvalidVoteRule.into());
            }

            let rule_info = next_account_info(accounts_iter)?;
            let rule = get_rule_data(program_id, rule_info, &post.channel)?;
            let deny_vote_weight = if proposal_deny_option_info.key == proposal_option_info.key {
                0
            } else {
                deny_option_vote_weight.weight
            };
            if !rule.vote_config.is_approved(
                option_vote_weight.weight,
                Some(deny_vote_weight),
                max_rule_weight.weight,
            ) {
                proposal_option_data.vote_result = OptionVoteResult::Defeated;
                break;
            }
        }
        if proposal_option_data.vote_result != OptionVoteResult::Defeated {
            proposal_option_data.vote_result = OptionVoteResult::Succeeded;
            match &mut proposal.vote_type {
                VoteType::SingleChoice {} => {
                    if !proposal.winning_options.is_empty() {
                        // We cant have two winning options for a singe choice proposal, hence this will be defeated
                        proposal.state = ProposalState::Defeated
                    } else {
                        proposal.winning_options.push(proposal_option_data.index);
                    }
                }
                VoteType::MultiChoice {
                    max_winning_options,
                    ..
                } => {
                    proposal.winning_options.push(proposal_option_data.index);

                    if let Some(max_winning_options) = max_winning_options {
                        if (*max_winning_options as usize) < proposal.winning_options.len() {
                            proposal.state = ProposalState::Defeated
                        }
                    }
                }
            }
        }
        proposal.options_counted_count = proposal.options_counted_count.checked_add(1).unwrap();
    } else {
        return Err(ProgramError::InvalidAccountData);
    }

    post.serialize(&mut *post_account_info.data.borrow_mut())?;

    Ok(())
}

pub fn process_calculate_max_vote_weights(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    let mut post = deserialize_post_account(&post_account_info.data.borrow())?;

    if let PostType::Proposal(proposal) = &mut post.post_type {
        // Will overwrite previous max vote weight calculations
        proposal.max_vote_weights_calculated_at = Some(Clock::get()?.unix_timestamp);
        for vote_weight in &mut proposal.rules_max_vote_weight {
            let rule_info = next_account_info(accounts_iter)?;
            let rule_data = get_rule_data(program_id, rule_info, &post.channel)?;

            let mut sum: u64 = 0;
            for mint_weight in rule_data.vote_config.mint_weights {
                let mint_info = next_account_info(accounts_iter)?;
                if mint_info.key != &mint_weight.mint {
                    return Err(PostError::InvalidVoteMint.into());
                }
                let supply = get_spl_token_mint_supply(mint_info)?;
                sum = sum
                    .checked_add(supply.checked_mul(mint_weight.weight).unwrap())
                    .unwrap();
            }
            vote_weight.weight = sum;
        }
    } else {
        return Err(ProgramError::InvalidAccountData);
    }

    post.serialize(&mut *post_account_info.data.borrow_mut())?;

    Ok(())
}
