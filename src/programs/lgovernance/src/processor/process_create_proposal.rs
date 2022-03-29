use crate::{
    accounts::AccountType,
    error::GovernanceError,
    instruction::{create_proposal_option, CreateProposalOptionType},
    state::{
        enums::VoteTipping,
        governance::GovernanceV2,
        proposal::{get_proposal_address_seeds, CommonRuleConfig},
    },
    state::{
        enums::{InstructionExecutionFlags, ProposalState},
        proposal::{ProposalV2, VoteType},
        token_owner_record::get_token_owner_record_data_for_owner,
    },
};
use shared::{
    account::{
        check_account_owner, check_system_program, create_and_serialize_account_verify_with_bump,
        get_account_data,
    },
    content::ContentSource,
};
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

pub fn process_create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote_type: VoteType,
    rules_count: u8,
    source: ContentSource,
    bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account_info = next_account_info(accounts_iter)?;
    let governance_account_info = next_account_info(accounts_iter)?;
    let creator_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let governance_data = get_account_data::<GovernanceV2>(program_id, governance_account_info)?;

    if !proposal_account_info.data_is_empty() {
        return Err(GovernanceError::ProposalAlreadyExists.into());
    }

    check_account_owner(governance_account_info, program_id)?;
    check_system_program(system_account.key)?;

    if !creator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let timestamp = Clock::get()?.unix_timestamp;
    let rent = Rent::get()?;

    // Create proposal
    create_and_serialize_account_verify_with_bump(
        payer_account,
        proposal_account_info,
        &ProposalV2 {
            account_type: AccountType::Proposal,
            state: ProposalState::Draft,
            vote_type,
            governance: *governance_account_info.key,
            creator: *creator_info.key,
            signatories_count: 0,
            signatories_signed_off_count: 0,
            start_voting_at: None,
            draft_at: timestamp,
            signing_off_at: None,
            voting_at: None,
            voting_at_slot: None,
            voting_completed_at: None,
            executing_at: None,
            closed_at: None,
            execution_flags: InstructionExecutionFlags::None,
            deny_option: None,
            max_voting_time: None,
            max_vote_weights_calculated_at: None,
            vote_threshold_percentage: None,
            options_counted_count: 0,
            options_executed_count: 0,
            options_count: 0,
            winning_options: Vec::new(),
            defeated_options: Vec::new(),
            rules_count,
            rules_max_vote_weight: Vec::new(),

            source,
        },
        &get_proposal_address_seeds(
            governance_account_info.key,
            &governance_data.proposals_count.to_le_bytes(),
            &[bump_seed],
        ),
        program_id,
        system_account,
        &rent,
    )?;

    /* create_and_serialize_account_verify_with_bump(
        payer_account,
        post_account_info,
        &ProposalV2 {
            account_type: AccountType::Proposal,
            post_type: match post.post_type {
                CreateProposalType::InformationPost => PostType::InformationPost(InformationPost {
                    created_at: timestamp,
                    downvotes: 0,
                    upvotes: 0,
                }),
                CreateProposalType::Proposal { vote_type } => {

                    pt
                }
            },
            channel: *channel_account_info.key,
            hash: post.hash,
            source: post.source,
            creator: *payer_account.key,
            deleted: false,
        },
        &[&content_hash, &[post.post_bump_seed]],
        program_id,
        system_account,
        &rent,
    )?; */

    Ok(())
}
