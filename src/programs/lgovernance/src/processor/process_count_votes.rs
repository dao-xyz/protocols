use crate::{
    error::GovernanceError,
    state::{enums::ProposalState, proposal::VoteType},
    state::{
        enums::VoteTipping,
        proposal::{
            get_proposal_data, proposal_option::get_proposal_option_data, OptionVoteResult,
        },
        scopes::scope::{get_scope_data, get_scope_data_for_governance, VotePowerUnit},
    },
    tokens::spl_utils::get_spl_token_mint_supply,
};

use borsh::BorshSerialize;
use ltag::state::TagRecordFactoryAccount;
use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

// This process will do final vote count, or can be used for vote tipping
pub fn process_count_votes(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account_info = next_account_info(accounts_iter)?;
    let proposal_option_info = next_account_info(accounts_iter)?;
    let proposal_deny_option_info = next_account_info(accounts_iter)?;
    let mut proposal = get_proposal_data(program_id, proposal_account_info)?;
    let current_unix_timestamp = Clock::get()?.unix_timestamp;
    let mut deny_option = match proposal.deny_option {
        Some(key) => {
            if proposal_deny_option_info.key == proposal_option_info.key {
                None // The option is same as deny option, so we just ignore it
            } else {
                if &key != proposal_deny_option_info.key {
                    return Err(GovernanceError::InvalidDenyOptionForProposal.into());
                }
                let proposal_deny_option_data = get_proposal_option_data(
                    program_id,
                    proposal_deny_option_info,
                    proposal_account_info.key,
                )?;

                Some((proposal_deny_option_info, proposal_deny_option_data))
            }
        }
        None => None,
    };

    let mut proposal_option_data =
        get_proposal_option_data(program_id, proposal_option_info, proposal_account_info.key)?;

    if proposal_option_data.vote_result != OptionVoteResult::None {
        return Err(GovernanceError::VotesAlreadyCounted.into());
    }

    if proposal.max_vote_weights_calculated_at.is_none() {
        return Err(GovernanceError::MaxWeightsNotCalculated.into());
    }
    for (i, max_scope_weight) in proposal.scopes_max_vote_weight.iter().enumerate() {
        let option_vote_weight = proposal_option_data.vote_weights.get(i).unwrap();
        let deny_vote_weight = match &deny_option {
            Some((_, data)) => {
                let deny_option_vote_weight = data.vote_weights.get(i).unwrap();
                if max_scope_weight.scope != option_vote_weight.scope
                    || max_scope_weight.scope != deny_option_vote_weight.scope
                {
                    return Err(GovernanceError::InvalidVotescope.into());
                }
                deny_option_vote_weight.weight
            }
            None => 0,
        };

        let scope_info = next_account_info(accounts_iter)?;
        let scope = get_scope_data_for_governance(program_id, scope_info, &proposal.governance)?;

        // If not approved by one scope, proposal is defauted
        /*
        msg!(
            "RESULT {} {} {} {} {}",
            proposal_deny_option_info.key == proposal_option_info.key,
            proposal_option_data.vote_result == OptionVoteResult::Defeated,
            option_vote_weight.weight,
            max_scope_weight.weight,
            scope.config.vote_config.is_approved(
                option_vote_weight.weight,
                Some(deny_vote_weight),
                max_scope_weight.weight,
        )
        );
        */
        let has_vote_time_ended =
            proposal.has_vote_time_ended(&scope.config.time_config, current_unix_timestamp);

        // vote tipping should be done here
        if has_vote_time_ended {
            if !scope.config.vote_config.is_approved(
                option_vote_weight.weight,
                Some(deny_vote_weight),
                max_scope_weight.weight,
            ) {
                proposal_option_data.vote_result = OptionVoteResult::Defeated;
                break;
            } else if scope.config.vote_config.is_approved(
                deny_vote_weight,
                None,
                max_scope_weight.weight,
            ) {
                proposal_option_data.vote_result = OptionVoteResult::Defeated;
            }

            // will be succeeded later if vote result isi non
        } else {
            // Check that we can do vote tipping

            if proposal.vote_type != VoteType::SingleChoice
            // Tipping should not be allowed for opinion only proposals (surveys without rejection) to allow everybody's voice to be heard
            || proposal.deny_option.is_none()
            || proposal.options_count != 2
            {
                proposal_option_data.vote_result = OptionVoteResult::None;
                msg!("SKIP TIPPING");
                break;
            };

            match scope.config.vote_config.vote_tipping {
                VoteTipping::Disabled => {
                    continue;
                }
                VoteTipping::Strict => {
                    if scope.config.vote_config.is_approved(
                        option_vote_weight.weight,
                        Some(deny_vote_weight),
                        max_scope_weight.weight,
                    ) && scope.config.vote_config.is_approved(
                        option_vote_weight.weight,
                        Some(
                            max_scope_weight
                                .weight
                                .saturating_sub(option_vote_weight.weight),
                        ),
                        max_scope_weight.weight,
                    ) && proposal_option_data.vote_result == OptionVoteResult::None
                    {
                        proposal_option_data.vote_result = OptionVoteResult::Succeeded;

                        match &mut deny_option {
                            Some((info, data)) => {
                                if data.vote_result == OptionVoteResult::None {
                                    data.vote_result = OptionVoteResult::Defeated;
                                    proposal.defeated_options.push(data.index);
                                    data.serialize(&mut *info.data.borrow_mut())?;
                                }
                            }
                            None => {}
                        };
                    }
                }
                VoteTipping::Early => {
                    if scope.config.vote_config.is_approved(
                        option_vote_weight.weight,
                        Some(deny_vote_weight),
                        max_scope_weight.weight,
                    ) && proposal_option_data.vote_result == OptionVoteResult::None
                    {
                        proposal_option_data.vote_result = OptionVoteResult::Succeeded;
                    }
                }
            };
            if scope
                .config
                .vote_config
                .is_approved(deny_vote_weight, None, max_scope_weight.weight)
            {
                proposal_option_data.vote_result = OptionVoteResult::Defeated;
            }
        }
    }

    /*    if has_vote_time_ended {
        if proposal_option_data.vote_result != OptionVoteResult::Defeated {
            proposal_option_data.vote_result = OptionVoteResult::Succeeded;
        }
    }
    else { // vote time has not ended
        if proposal_option_data.vote_result != OptionVoteResult::Succeeded
        {
            proposal_option_data.vote_result = OptionVoteResult::None
        }
    } */
    // So if not defeated, it must have succeeded
    if proposal_option_data.vote_result == OptionVoteResult::Succeeded {
        proposal.winning_options.push(proposal_option_data.index);
    } else if proposal_option_data.vote_result == OptionVoteResult::Defeated {
        proposal.defeated_options.push(proposal_option_data.index)
    }

    // Try finalize
    msg!(
        "{} {} {}",
        proposal.winning_options.len(),
        proposal.defeated_options.len(),
        proposal.options_count
    ); // OPTION SUCCEEDDED BUT NOT PROPOSAL?

    if (proposal.defeated_options.len() + proposal.winning_options.len()) as u16
        == proposal.options_count
    {
        // Done
        match &mut proposal.vote_type {
            VoteType::SingleChoice {} => {
                if proposal.winning_options.len() != 1 {
                    // We cant have two winning options for a singe choice proposal, hence this will be defeated
                    proposal.set_completed_voting_state(
                        ProposalState::Defeated,
                        current_unix_timestamp,
                    );
                } else if let Some((_, data)) = &deny_option {
                    if proposal.winning_options[0] == data.index {
                        proposal.set_completed_voting_state(
                            ProposalState::Defeated,
                            current_unix_timestamp,
                        );
                    } else {
                        proposal.set_completed_voting_state(
                            ProposalState::Succeeded,
                            current_unix_timestamp,
                        );
                    }
                } else {
                    proposal.set_completed_voting_state(
                        ProposalState::Succeeded,
                        current_unix_timestamp,
                    );
                }
            }
            VoteType::MultiChoice {
                max_winning_options,
                ..
            } => {
                if let Some(max_winning_options) = max_winning_options {
                    if (*max_winning_options as usize) < proposal.winning_options.len() {
                        proposal.state = ProposalState::Defeated
                    } else {
                        // check if deny option is in winning options
                        if let Some((_, data)) = deny_option {
                            if proposal
                                .winning_options
                                .iter()
                                .any(|option| option == &data.index)
                            {
                                // Deny option can not win and also some other option
                                proposal.set_completed_voting_state(
                                    ProposalState::Defeated,
                                    current_unix_timestamp,
                                );
                            } else {
                                proposal.set_completed_voting_state(
                                    ProposalState::Succeeded,
                                    current_unix_timestamp,
                                );
                            }
                        } else {
                            proposal.set_completed_voting_state(
                                ProposalState::Succeeded,
                                current_unix_timestamp,
                            );
                        }
                    }
                }
            }
        }
    }
    /*
    msg!(
        "XXXX {} {} {}",
        proposal_deny_option_info.key == proposal_option_info.key,
        proposal_option_data.vote_result != OptionVoteResult::Defeated
    );
    */

    /* if !has_vote_time_ended && proposal_option_data.vote_result == OptionVoteResult::Defeated {
        proposal_option_data.vote_result = OptionVoteResult::None // Set to none, since we can not defeat before vote time ending
    } */

    proposal.options_counted_count = proposal.options_counted_count.checked_add(1).unwrap();
    proposal.serialize(&mut *proposal_account_info.data.borrow_mut())?;
    proposal_option_data.serialize(&mut *proposal_option_info.data.borrow_mut())?;

    Ok(())
}

pub fn process_count_max_vote_weights(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account_info = next_account_info(accounts_iter)?;
    let mut proposal = get_proposal_data(program_id, proposal_account_info)?;
    proposal.max_vote_weights_calculated_at = Some(Clock::get()?.unix_timestamp);

    for vote_weight in &mut proposal.scopes_max_vote_weight {
        let scope_info = next_account_info(accounts_iter)?;

        let scope_data = get_scope_data(program_id, scope_info)?;

        let mut sum: u64 = 0;
        for source_weight in scope_data.config.vote_config.source_weights {
            let supply = match &source_weight.source {
                VotePowerUnit::Tag { record_factory } => {
                    let record_factory_info = next_account_info(accounts_iter)?;

                    if record_factory_info.key != record_factory {
                        return Err(GovernanceError::InvalidVoteMint.into());
                    }
                    get_account_data::<TagRecordFactoryAccount>(&ltag::id(), record_factory_info)?
                        .outstanding_records
                }
                VotePowerUnit::Mint(mint) => {
                    let mint_info = next_account_info(accounts_iter)?;

                    if mint_info.key != mint {
                        return Err(GovernanceError::InvalidVoteMint.into());
                    }
                    get_spl_token_mint_supply(mint_info)?
                }
            };
            sum = sum
                .checked_add(supply.checked_mul(source_weight.weight).unwrap())
                .unwrap();
        }
        vote_weight.weight = sum;
    }

    proposal.serialize(&mut *proposal_account_info.data.borrow_mut())?;

    Ok(())
}
