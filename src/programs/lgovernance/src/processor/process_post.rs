use crate::{
    accounts::AccountType,
    instruction::{create_proposal_option, CreatePost, CreatePostType, CreateProposalOptionType},
    state::post::{
        InformationPost, PostAccount, PostType,
    },
    state::{
        enums::{InstructionExecutionFlags, ProposalState},
        proposal::{ProposalV2},
        rules::rule_weight::RuleWeight,
        token_owner_record::get_token_owner_record_data_for_owner,
    },
};
use lchannel::state::ChannelAccount;
use shared::account::{
    check_account_owner, check_system_program, create_and_serialize_account_verify_with_bump,
    get_account_data,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_post(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    post: CreatePost,
) -> ProgramResult {
    let timestamp = Clock::get()?.unix_timestamp;
    let accounts_iter = &mut accounts.iter();
    let post_account_info = next_account_info(accounts_iter)?;
    check_account_owner(post_account_info, program_id)?;
    let channel_account_info = next_account_info(accounts_iter)?;
    check_account_owner(channel_account_info, program_id)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let content_hash = post.hash;
    let rent = Rent::get()?;

    check_system_program(system_account.key)?;
    create_and_serialize_account_verify_with_bump(
        payer_account,
        post_account_info,
        &PostAccount {
            account_type: AccountType::Post,
            post_type: match post.post_type {
                CreatePostType::InformationPost => PostType::InformationPost(InformationPost {
                    created_at: timestamp,
                    downvotes: 0,
                    upvotes: 0,
                }),
                CreatePostType::Proposal { vote_type } => {
                    let proposal_owner_record_info = next_account_info(accounts_iter)?;
                    let governance_authority_info = next_account_info(accounts_iter)?;
                    let proposal_deny_option_info = next_account_info(accounts_iter)?;

                    get_account_data::<ChannelAccount>(program_id, channel_account_info)?;

                    //   let rent_info = next_account_info(accounts_iter)?;

                    let mut proposal_owner_record_data = get_token_owner_record_data_for_owner(
                        program_id,
                        proposal_owner_record_info,
                        governance_authority_info,
                    )?;

                    proposal_owner_record_data
                        .assert_token_owner_or_delegate_is_signer(governance_authority_info)?;

                    /*       let voter_weight = proposal_owner_record_data.resolve_voter_weight(
                                           program_id,
                                           account_info_iter,
                                           realm_info.key,
                                           &realm_data,
                                           VoterWeightAction::CreateProposal,
                                           governance_info.key,
                                       )?;
                    */
                    // Ensure proposal owner (TokenOwner) has enough tokens to create proposal and no outstanding proposals
                    proposal_owner_record_data.assert_can_create_proposal(
                      /*   &realm_data,
                        &governance_data.config,
                        voter_weight, */
                    )?;

                    proposal_owner_record_data.outstanding_proposal_count =
                        proposal_owner_record_data
                            .outstanding_proposal_count
                            .checked_add(1)
                            .unwrap();
                    proposal_owner_record_data
                        .serialize(&mut *proposal_owner_record_info.data.borrow_mut())?;

                    let pt = PostType::Proposal(ProposalV2 {
                        state: ProposalState::Draft,
                        token_owner_record: *proposal_owner_record_info.key,
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
                        vote_type,
                        deny_option_exist: true,
                        max_voting_time: None,
                        max_vote_weights_calculated_at: None,
                        vote_threshold_percentage: None,
                        options_counted_count: 0,
                        options_executed_count: 0,
                        options_count: 0,
                        winning_options: Vec::new(),
                        rules_max_vote_weight: post
                            .rules
                            .iter()
                            .map(|rule| {
                                RuleWeight {
                                    rule: *rule,
                                    weight: 0,
                                }
                            })
                            .collect::<Vec<RuleWeight>>(),
                    });
                    // create deny option
                    invoke(
                        &create_proposal_option(
                            program_id,
                            payer_account.key,
                            post_account_info.key,
                            CreateProposalOptionType::Deny,
                            0,
                        ),
                        &[
                            proposal_deny_option_info.clone(),
                            post_account_info.clone(),
                            payer_account.clone(),
                            system_account.clone(),
                        ],
                    )?;
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
    )?;

    Ok(())
}
