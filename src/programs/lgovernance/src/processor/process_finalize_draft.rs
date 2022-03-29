//! Program state processor

use crate::{
    error::GovernanceError,
    state::{
        enums::ProposalState, governance::get_governance_data,
        proposal::get_proposal_data_for_creator, rules::rule::get_rule_data_for_governance,
    },
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// Processes DepositGoverningTokens instruction
pub fn process_finalize_draft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // verify proposal
    let account_info_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_info_iter)?;
    let governance_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;

    let mut proposal_data = get_proposal_data_for_creator(program_id, proposal_info, creator_info)?;
    msg!("X");

    proposal_data.assert_can_finalize_draft(creator_info)?;

    if proposal_data.rules_count != proposal_data.rules_max_vote_weight.len() as u8 {
        return Err(GovernanceError::MissingRulesForProposal.into());
    }
    if &proposal_data.governance != governance_info.key {
        return Err(GovernanceError::InvalidGovernanceForProposal.into());
    }
    msg!("XX");

    let mut governance_data = get_governance_data(program_id, governance_info)?;
    msg!("XXX");
    for rule_weight in &proposal_data.rules_max_vote_weight {
        let rule_info = next_account_info(account_info_iter)?;
        if rule_info.key != &rule_weight.rule {
            return Err(GovernanceError::InvalidVoteRule.into());
        }
        let rule = get_rule_data_for_governance(program_id, rule_info, &proposal_data.governance)?;
        rule.config.proposal_config.assert_can_create_proposal(
            program_id,
            &proposal_data,
            account_info_iter,
        )?;
    }
    msg!("XXXX");

    let clock = Clock::get()?;

    proposal_data.voting_at = Some(clock.unix_timestamp);
    proposal_data.voting_at_slot = Some(clock.slot);
    proposal_data.state = ProposalState::Voting;

    proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;

    governance_data.proposals_count = governance_data.proposals_count.checked_add(1).unwrap();
    governance_data.serialize(&mut *governance_info.data.borrow_mut())?;

    Ok(())
    /*
    proposal_owner_record_data.assert_can_create_proposal(
        /*   &realm_data,
          &governance_data.config,
          voter_weight, */
      )?;

    proposal_owner_record_data.outstanding_proposal_count = proposal_owner_record_data
        .outstanding_proposal_count
        .checked_add(1)
        .unwrap();
    proposal_owner_record_data.serialize(&mut *proposal_owner_record_info.data.borrow_mut())?; */

    /*


    proposal_owner_record_data
        .assert_token_owner_or_delegate_is_signer(governance_authority_info)?;

    let mut proposal_owner_record_data = get_token_owner_record_data_for_owner(
           program_id,
           proposal_owner_record_info,
           governance_authority_info,
       )?;
    */
    // check all rules conditions of creating a proposal are met
    // SET state of proposal to verified
}
